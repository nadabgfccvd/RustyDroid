//! `Resources.NotFoundException` is a `RuntimeException` on AOSP, so guest code
//! can catch and rethrow it like any unchecked exception.
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): `android_res::not_found`
//! threw `Resources$NotFoundException`, but its class had no hierarchy entry.
//! The coroutine machinery then did `checkcast Throwable` on it, got
//! `ClassCastException: Resources$NotFoundException cannot be cast to Throwable`,
//! and the process died with `FATAL EXCEPTION: main`.

mod common;

use common::*;
use rd_vm::value::Value;

const NOT_FOUND: &str = "Landroid/content/res/Resources$NotFoundException;";
const THROWABLE: &str = "Ljava/lang/Throwable;";

/// `Object asThrowable(Object)`: `return (Throwable) arg;` (check-cast, return-object).
fn probe_engine() -> rd_vm::Engine {
    let mut b = DexBuilder::new();
    let cls = b.class("LNotFoundProbe;", "Ljava/lang/Object;");
    let throwable_t = b.type_idx(THROWABLE);
    let mut code = op21c(0x1f, 0, throwable_t); // check-cast v0, Throwable
    code.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "asThrowable",
        "Ljava/lang/Object;",
        vec!["Ljava/lang/Object;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 1, 1, code)),
    );
    engine_of(&b)
}

#[test]
fn resources_not_found_is_a_throwable_for_checkcast() {
    let mut e = probe_engine();
    let exc = e
        .heap
        .alloc_instance(NOT_FOUND.to_string(), Vec::new())
        .expect("alloc NotFoundException");
    let r = e
        .invoke_static(
            "LNotFoundProbe;",
            "asThrowable",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            &[Value::Obj(exc)],
        )
        .expect("checkcast Throwable must succeed for Resources.NotFoundException");
    assert_eq!(r, Value::Obj(exc));
}
