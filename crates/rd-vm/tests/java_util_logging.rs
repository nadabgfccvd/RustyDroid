//! `java.util.logging.Logger`, string-level subset (see `java_util_logging.rs`).
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): the Guava
//! `AbstractFuture` in the APK calls `Logger.getLogger(name)` from its static
//! initializer. The test pins what that initializer relies on: one logger per
//! name, `getName`, and the records above the root level going through without
//! an error. Records below INFO are dropped (the root default).

mod common;

use common::*;
use rd_vm::value::Value;

const LOGGER: &str = "Ljava/util/logging/Logger;";

fn engine() -> rd_vm::Engine {
    engine_of(&DexBuilder::new())
}

fn logger(e: &mut rd_vm::Engine, name: &str) -> Value {
    let s = e.heap.alloc_string(name.to_string()).expect("alloc name");
    e.invoke_static(
        LOGGER,
        "getLogger",
        "(Ljava/lang/String;)Ljava/util/logging/Logger;",
        &[Value::Obj(s)],
    )
    .expect("Logger.getLogger")
}

#[test]
fn get_logger_returns_one_object_per_name() {
    let mut e = engine();
    let a = logger(&mut e, "a1.h");
    let b = logger(&mut e, "a1.h");
    let c = logger(&mut e, "other");
    assert!(matches!(a, Value::Obj(_)), "got {a:?}");
    assert!(ref_eq(&a, &b), "same name, same logger");
    assert!(!ref_eq(&a, &c), "different name, different logger");
}

#[test]
fn get_name_returns_the_name_it_was_created_with() {
    let mut e = engine();
    let l = logger(&mut e, "a1.h");
    let Value::Obj(r) = e
        .invoke_instance(obj_of(&l), "getName", "()Ljava/lang/String;", &[])
        .expect("getName")
    else {
        panic!("getName returned no string");
    };
    assert_eq!(e.heap.as_str(r).expect("string"), "a1.h");
}

#[test]
fn severe_and_info_are_delivered_and_fine_is_dropped() {
    let mut e = engine();
    let l = logger(&mut e, "a1.h");
    let me = obj_of(&l);
    let msg = e.heap.alloc_string("boom".to_string()).expect("msg");
    for m in ["severe", "warning", "info"] {
        e.invoke_instance(me, m, "(Ljava/lang/String;)V", &[Value::Obj(msg)])
            .unwrap_or_else(|err| panic!("{m} failed: {err:?}"));
    }
    e.invoke_instance(me, "fine", "(Ljava/lang/String;)V", &[Value::Obj(msg)])
        .expect("fine is accepted and dropped by the root level");
}

fn obj_of(v: &Value) -> u32 {
    match v {
        Value::Obj(r) => *r,
        other => panic!("not an object: {other:?}"),
    }
}

fn ref_eq(a: &Value, b: &Value) -> bool {
    matches!((a, b), (Value::Obj(x), Value::Obj(y)) if x == y)
}
