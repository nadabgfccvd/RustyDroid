//! Apêndice D (S12) — RxJava 3 como `dex-passthrough`: o DEX da biblioteca
//! executa na VM, sem shim host.
//!
//! O DEX de `tests/fixtures/rxjava3/classes.dex` é a biblioteca real
//! (io.reactivex.rxjava3:rxjava 3.1.10 + org.reactivestreams 1.0.4) reduzida
//! por R8 com as regras em `keep.pro`. Regenerar: `scripts/gen-rxjava3-fixture.sh`.
//! O caller é um DEX sintético (`DexBuilder`) que chama a API pública pelo
//! mesmo caminho de bytecode de um APK: `invoke-static`/`invoke-virtual`/
//! `invoke-interface`. Os dois DEX entram no `Engine` como um APK com
//! classes.dex próprio mais a biblioteca.

mod common;

use common::{op11x, op35c, DexBuilder, ACC_PUBLIC};
use rd_vm::engine::VmConfig;
use rd_vm::value::Value;
use rd_vm::Engine;

const CALLER: &str = "LRxCaller;";
const OBSERVABLE: &str = "Lio/reactivex/rxjava3/core/Observable;";
const DISPOSABLE: &str = "Lio/reactivex/rxjava3/disposables/Disposable;";
const RX_DEX: &[u8] = include_bytes!("fixtures/rxjava3/classes.dex");

// opcodes (dex-format)
const INVOKE_VIRTUAL: u8 = 0x6E;
const INVOKE_STATIC: u8 = 0x71;
const INVOKE_INTERFACE: u8 = 0x72;
const MOVE_RESULT: u8 = 0x0A;
const MOVE_RESULT_OBJECT: u8 = 0x0C;
const RETURN: u8 = 0x0F;

/// Caller sintético com quatro métodos estáticos que usam só a API pública do RxJava.
fn caller_dex() -> DexBuilder {
    let mut b = DexBuilder::new();
    let cls = b.class(CALLER, "Ljava/lang/Object;");

    let p_obj = b.proto_idx("Z", vec!["Ljava/lang/Object;".to_string()]);
    let p_none = b.proto_idx("Z", vec![]);
    let p_obs_obj = b.proto_idx(OBSERVABLE, vec!["Ljava/lang/Object;".to_string()]);
    let m_just = b.method_idx(OBSERVABLE, p_obs_obj, "just");
    let p_obs = b.proto_idx(OBSERVABLE, vec![]);
    let m_never = b.method_idx(OBSERVABLE, p_obs, "never");
    let m_empty = b.method_idx(OBSERVABLE, p_obs, "empty");
    let p_disp = b.proto_idx(DISPOSABLE, vec![]);
    let m_subscribe = b.method_idx(OBSERVABLE, p_disp, "subscribe");
    let v_void = b.proto_idx("V", vec![]);
    let m_dispose = b.method_idx(DISPOSABLE, v_void, "dispose");
    let m_is_disposed = b.method_idx(DISPOSABLE, p_none, "isDisposed");
    let _ = p_obj;

    // justCompletes(Object) -> boolean: Observable.just(x).subscribe().isDisposed()
    let mut body = op35c(INVOKE_STATIC, 1, m_just, [0, 0, 0, 0, 0]);
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_VIRTUAL, 1, m_subscribe, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_INTERFACE, 1, m_is_disposed, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT, 0));
    body.extend(op11x(RETURN, 0));
    let code = b.code(1, 1, 1, body);
    b.direct(
        cls,
        "justCompletes",
        "Z",
        vec!["Ljava/lang/Object;"],
        ACC_PUBLIC | 0x8,
        Some(code),
    );

    // neverStaysOpen() -> boolean: Observable.never().subscribe().isDisposed()
    let mut body = op35c(INVOKE_STATIC, 0, m_never, [0, 0, 0, 0, 0]);
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_VIRTUAL, 1, m_subscribe, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_INTERFACE, 1, m_is_disposed, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT, 0));
    body.extend(op11x(RETURN, 0));
    let code = b.code(1, 0, 1, body);
    b.direct(
        cls,
        "neverStaysOpen",
        "Z",
        vec![],
        ACC_PUBLIC | 0x8,
        Some(code),
    );

    // neverDisposedByDispose() -> boolean: never().subscribe(); dispose(); isDisposed()
    let mut body = op35c(INVOKE_STATIC, 0, m_never, [0, 0, 0, 0, 0]);
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_VIRTUAL, 1, m_subscribe, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_INTERFACE, 1, m_dispose, [0, 0, 0, 0, 0]));
    body.extend(op35c(INVOKE_INTERFACE, 1, m_is_disposed, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT, 0));
    body.extend(op11x(RETURN, 0));
    let code = b.code(1, 0, 1, body);
    b.direct(
        cls,
        "neverDisposedByDispose",
        "Z",
        vec![],
        ACC_PUBLIC | 0x8,
        Some(code),
    );

    // emptyCompletes() -> boolean: Observable.empty().subscribe().isDisposed()
    let mut body = op35c(INVOKE_STATIC, 0, m_empty, [0, 0, 0, 0, 0]);
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_VIRTUAL, 1, m_subscribe, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT_OBJECT, 0));
    body.extend(op35c(INVOKE_INTERFACE, 1, m_is_disposed, [0, 0, 0, 0, 0]));
    body.extend(op11x(MOVE_RESULT, 0));
    body.extend(op11x(RETURN, 0));
    let code = b.code(1, 0, 1, body);
    b.direct(
        cls,
        "emptyCompletes",
        "Z",
        vec![],
        ACC_PUBLIC | 0x8,
        Some(code),
    );

    b
}

fn engine_with_rxjava() -> Engine {
    let caller = rd_dex::Dex::parse(caller_dex().finish()).expect("caller DEX parseia");
    let rx = rd_dex::Dex::parse(RX_DEX.to_vec()).expect("DEX do RxJava parseia");
    Engine::new(vec![caller, rx], VmConfig::default())
}

fn call_bool(e: &mut Engine, name: &str, sig: &str, args: &[Value]) -> bool {
    match e
        .invoke_static(CALLER, name, sig, args)
        .expect("chamada ao RxJava")
    {
        Value::Int(n) => n != 0,
        other => panic!("retorno inesperado: {other:?}"),
    }
}

#[test]
fn just_then_subscribe_completes_and_is_disposed() {
    let mut e = engine_with_rxjava();
    let s = e.heap.alloc_string("x".to_string()).expect("string");
    assert!(
        call_bool(
            &mut e,
            "justCompletes",
            "(Ljava/lang/Object;)Z",
            &[Value::Obj(s)]
        ),
        "Observable.just completa, então o Disposable do subscribe() fica disposed"
    );
}

#[test]
fn never_stays_open_after_subscribe() {
    let mut e = engine_with_rxjava();
    assert!(
        !call_bool(&mut e, "neverStaysOpen", "()Z", &[]),
        "Observable.never não completa: o Disposable não fica disposed"
    );
}

#[test]
fn dispose_marks_the_subscription_disposed() {
    let mut e = engine_with_rxjava();
    assert!(
        call_bool(&mut e, "neverDisposedByDispose", "()Z", &[]),
        "dispose() no Disposable devolvido por subscribe() marca isDisposed"
    );
}

#[test]
fn empty_completes_immediately() {
    let mut e = engine_with_rxjava();
    assert!(call_bool(&mut e, "emptyCompletes", "()Z", &[]));
}

#[test]
fn library_dex_is_what_runs_without_it_the_call_fails() {
    // Sem o DEX da biblioteca no Engine, a mesma chamada não resolve:
    // o resultado acima vem do bytecode do RxJava, não de um shim.
    let caller = rd_dex::Dex::parse(caller_dex().finish()).expect("caller");
    let mut e = Engine::new(vec![caller], VmConfig::default());
    assert!(e
        .invoke_static(CALLER, "emptyCompletes", "()Z", &[])
        .is_err());
}
