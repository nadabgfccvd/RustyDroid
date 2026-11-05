//! `Context.getMainExecutor()` (MEDIA-03, scanner do Comprix).
//!
//! Antes: o host não implementava o método; o `ContextCompat`/CameraX do app
//! recebia `null`, o `Intrinsics.checkNotNull` falhava e a composição da tela
//! de escaneamento quebrava sem aviso. Agora o método devolve um
//! `android.os.HandlerExecutor` (implementa `java/util/concurrent/Executor`),
//! e `execute(Runnable)` enfileira no main looper.

mod common;

use common::*;
use rd_vm::value::Value;

#[test]
fn get_main_executor_returns_an_executor_that_is_a_subtype_of_executor() {
    let mut e = engine_of(&DexBuilder::new());
    let app = e
        .heap
        .alloc_instance("Landroid/app/Application;".into(), Vec::new())
        .expect("app");

    let Value::Obj(exec) = e
        .invoke_instance(
            app,
            "getMainExecutor",
            "()Ljava/util/concurrent/Executor;",
            &[],
        )
        .expect("getMainExecutor não pode falhar")
    else {
        panic!("getMainExecutor deve devolver um objeto, não null");
    };

    assert_eq!(
        e.heap.class_of(exec).unwrap(),
        "Landroid/os/HandlerExecutor;"
    );
    // A causa da ClassCastException: o checkcast para Executor precisa passar.
    assert!(e.cp.is_subtype(
        "Landroid/os/HandlerExecutor;",
        "Ljava/util/concurrent/Executor;"
    ));
}

#[test]
fn main_executor_execute_enqueues_runnable_and_rejects_null() {
    let mut e = engine_of(&DexBuilder::new());
    let app = e
        .heap
        .alloc_instance("Landroid/app/Application;".into(), Vec::new())
        .expect("app");
    let Value::Obj(exec) = e
        .invoke_instance(
            app,
            "getMainExecutor",
            "()Ljava/util/concurrent/Executor;",
            &[],
        )
        .unwrap()
    else {
        panic!("getMainExecutor deve devolver um objeto");
    };

    // Runnable real (Thread implementa Runnable na hierarquia da VM).
    let runnable = e
        .heap
        .alloc_instance("Ljava/lang/Thread;".into(), Vec::new())
        .expect("runnable");
    e.invoke_instance(
        exec,
        "execute",
        "(Ljava/lang/Runnable;)V",
        &[Value::Obj(runnable)],
    )
    .expect("execute com runnable deve enfileirar");

    // Sem runnable válido: erro tipado, não silêncio.
    let err = e.invoke_instance(exec, "execute", "(Ljava/lang/Runnable;)V", &[Value::Null]);
    assert!(err.is_err(), "execute(null) deve falhar de forma tipada");
}
