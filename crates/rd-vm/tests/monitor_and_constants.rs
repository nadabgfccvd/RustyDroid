//! Regressão para a semântica de monitores e de constantes de classe/tipo.
//!
//! - `monitor-exit` sem `monitor-enter` da mesma thread lança
//!   `IllegalMonitorStateException`; enters balanceados seguem válidos.
//! - `monitor-enter` em `null` continua `NullPointerException`.
//! - `const-class` aceita descritores primitivos (`I`, `V`, …), como a JVM.
//! - `const-method-type` (0xFF) produz um `MethodType` com o descritor do proto.

use rd_testsupport::vm::*;
use rd_vm::{heap::HeapObj, Engine, Value, VmExit};

const IMSE: &str = "Ljava/lang/IllegalMonitorStateException;";
const NPE: &str = "Ljava/lang/NullPointerException;";
const MONITOR_CLS: &str = "LMonTest;";
const LOCK_CLS: &str = "LLock;";

/// Monta `static run()<ret>` com `units` e devolve a engine pronta.
fn engine_with_run(
    ret: &str,
    regs: u16,
    build: impl FnOnce(&mut DexBuilder) -> Vec<u16>,
) -> Engine {
    let mut b = DexBuilder::new();
    let cls = b.class(MONITOR_CLS, "Ljava/lang/Object;");
    let _lock = b.class(LOCK_CLS, "Ljava/lang/Object;");
    let units = build(&mut b);
    b.direct(
        cls,
        "run",
        ret,
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(regs, 0, 0, units)),
    );
    engine_of(&b)
}

/// `new-instance v0, LLock;` seguido dos `extra` (monitores etc.).
fn with_new_lock(b: &mut DexBuilder, extra: Vec<u16>) -> Vec<u16> {
    let lock_t = b.type_idx(LOCK_CLS);
    let mut units = op21c(0x22, 0, lock_t); // new-instance v0
    units.extend(extra);
    units
}

fn exception_class(r: Result<Value, VmExit>) -> String {
    match r {
        Err(VmExit::Exception(t)) => t.class,
        other => panic!("esperava exceção, veio {other:?}"),
    }
}

fn run_int(e: &mut Engine) -> i32 {
    match e.invoke_static(MONITOR_CLS, "run", "()I", &[]) {
        Ok(Value::Int(n)) => n,
        other => panic!("esperava int, veio {other:?}"),
    }
}

#[test]
fn monitor_exit_without_enter_throws_illegal_monitor_state() {
    let mut e = engine_with_run("I", 1, |b| {
        with_new_lock(
            b,
            [op11x(0x1E, 0), op11n(0x12, 0, 0), op11x(0x0F, 0)]
                .into_iter()
                .flatten()
                .collect(),
        )
    });
    assert_eq!(
        exception_class(e.invoke_static(MONITOR_CLS, "run", "()I", &[])),
        IMSE
    );
}

#[test]
fn balanced_nested_monitors_are_valid_and_release_the_lock() {
    let mut e = engine_with_run("I", 1, |b| {
        with_new_lock(
            b,
            [
                op11x(0x1D, 0), // monitor-enter v0
                op11x(0x1D, 0), // monitor-enter v0 (aninhado)
                op11x(0x1E, 0), // monitor-exit v0
                op11x(0x1E, 0), // monitor-exit v0
                op11x(0x1E, 0), // exit extra: já liberado → IMSE
            ]
            .into_iter()
            .flatten()
            .collect(),
        )
    });
    // Após liberar, um exit extra precisa falhar (o monitor não está mantido).
    assert_eq!(
        exception_class(e.invoke_static(MONITOR_CLS, "run", "()I", &[])),
        IMSE
    );
}

#[test]
fn balanced_monitor_pair_returns_normally() {
    let mut e = engine_with_run("I", 2, |b| {
        let lock_t = b.type_idx(LOCK_CLS);
        let mut units = op21c(0x22, 0, lock_t); // new-instance v0
        units.extend(op11x(0x1D, 0)); // monitor-enter v0
        units.extend(op11x(0x1E, 0)); // monitor-exit v0
        units.extend(op21s(0x13, 1, 77)); // const/16 v1, 77 (const/4 só cabe em 4 bits)
        units.extend(op11x(0x0F, 1)); // return v1
        units
    });
    assert_eq!(run_int(&mut e), 77);
}

#[test]
fn monitor_enter_on_null_is_still_npe() {
    let mut e = engine_with_run("I", 1, |_b| {
        [
            op11n(0x12, 0, 0),
            op11x(0x1D, 0),
            op11n(0x12, 0, 1),
            op11x(0x0F, 0),
        ]
        .into_iter()
        .flatten()
        .collect()
    });
    assert_eq!(
        exception_class(e.invoke_static(MONITOR_CLS, "run", "()I", &[])),
        NPE
    );
}

#[test]
fn const_class_accepts_primitive_descriptors() {
    let mut b = DexBuilder::new();
    let cls = b.class(MONITOR_CLS, "Ljava/lang/Object;");
    let int_t = b.type_idx("I");
    let mut units = op21c(0x1C, 0, int_t); // const-class v0, I
    units.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "run",
        "Ljava/lang/Class;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, units)),
    );
    let mut e = engine_of(&b);
    let r = match e.invoke_static(MONITOR_CLS, "run", "()Ljava/lang/Class;", &[]) {
        Ok(Value::Obj(r)) => r,
        other => panic!("const-class I deveria devolver um Class, veio {other:?}"),
    };
    assert_eq!(
        e.heap.get_field(r, "class_name").map(|v| match v {
            Value::Obj(s) => match e.heap.get(s) {
                Ok(HeapObj::Str(t)) => t.clone(),
                other => panic!("class_name não é string: {other:?}"),
            },
            other => panic!("class_name inesperado: {other:?}"),
        }),
        Ok("I".to_string())
    );
}

#[test]
fn const_method_type_builds_a_method_type_with_the_proto_descriptor() {
    let mut b = DexBuilder::new();
    let cls = b.class(MONITOR_CLS, "Ljava/lang/Object;");
    let proto = b.proto_idx("V", vec!["I".to_string()]);
    let mut units = op21c(0xFF, 0, proto as u16); // const-method-type v0, proto
    units.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "run",
        "Ljava/lang/invoke/MethodType;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, units)),
    );
    let mut e = engine_of(&b);
    let r = match e.invoke_static(MONITOR_CLS, "run", "()Ljava/lang/invoke/MethodType;", &[]) {
        Ok(Value::Obj(r)) => r,
        other => panic!("const-method-type deveria devolver um MethodType, veio {other:?}"),
    };
    assert_eq!(e.heap.class_of(r).unwrap(), "Ljava/lang/invoke/MethodType;");
    let Ok(Value::Obj(d)) = e.heap.get_field(r, "descriptor") else {
        panic!("MethodType sem descriptor");
    };
    let Ok(HeapObj::Str(desc)) = e.heap.get(d) else {
        panic!("descriptor não é string");
    };
    assert_eq!(desc, "(I)V");
}
