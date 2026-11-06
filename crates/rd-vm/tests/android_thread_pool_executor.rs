//! `java.util.concurrent.ThreadPoolExecutor` (CameraX constructs one directly).
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): after the camera
//! configuration was found, `new ThreadPoolExecutor(...)` answered `NOT_FOUND`
//! because the class had no model. The VM is single-threaded, so the instance
//! is the same inline `ExecutorService` the `Executors` factories return; these
//! tests pin the constructor overloads and the subtype relation, and check that
//! a non-JDK signature is still refused.

mod common;

use common::*;
use rd_vm::value::Value;

const TPE: &str = "Ljava/util/concurrent/ThreadPoolExecutor;";
const CTOR_JDK: &str = "(IIJLjava/util/concurrent/TimeUnit;Ljava/util/concurrent/BlockingQueue;)V";

/// `static boolean isExecutor(Object)`: `return arg instanceof ExecutorService;`
fn probe_engine() -> rd_vm::Engine {
    let mut b = DexBuilder::new();
    let cls = b.class("LTpeProbe;", "Ljava/lang/Object;");
    let svc_t = b.type_idx("Ljava/util/concurrent/ExecutorService;");
    let mut code = op22c(0x20, 0, 0, svc_t); // instance-of v0, v0, ExecutorService
    code.extend(op11x(0x0f, 0)); // return v0
    b.direct(
        cls,
        "isExecutor",
        "Z",
        vec!["Ljava/lang/Object;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 1, 1, code)),
    );
    engine_of(&b)
}

#[test]
fn jdk_constructor_builds_an_executor_service() {
    let mut e = probe_engine();
    let tpe = e
        .heap
        .alloc_instance(TPE.to_string(), Vec::new())
        .expect("alloc ThreadPoolExecutor");
    e.invoke_instance(
        tpe,
        "<init>",
        CTOR_JDK,
        &[
            Value::Int(1),
            Value::Int(4),
            Value::Long(60),
            Value::Null,
            Value::Null,
        ],
    )
    .expect("ThreadPoolExecutor(int,int,long,TimeUnit,BlockingQueue)");
    let is = e
        .invoke_static(
            "LTpeProbe;",
            "isExecutor",
            "(Ljava/lang/Object;)Z",
            &[Value::Obj(tpe)],
        )
        .expect("instance-of through bytecode");
    assert_eq!(
        is,
        Value::Int(1),
        "ThreadPoolExecutor is an ExecutorService"
    );
}

/// The configuration setters CameraX calls are accepted (the inline executor
/// does not apply them), and their getters are not invented.
#[test]
fn configuration_setters_are_accepted_and_getters_are_not_invented() {
    let mut e = probe_engine();
    let tpe = e
        .heap
        .alloc_instance(TPE.to_string(), Vec::new())
        .expect("alloc ThreadPoolExecutor");
    e.invoke_instance(
        tpe,
        "<init>",
        CTOR_JDK,
        &[
            Value::Int(1),
            Value::Int(4),
            Value::Long(60),
            Value::Null,
            Value::Null,
        ],
    )
    .expect("ctor");
    e.invoke_instance(
        tpe,
        "setRejectedExecutionHandler",
        "(Ljava/util/concurrent/RejectedExecutionHandler;)V",
        &[Value::Null],
    )
    .expect("setRejectedExecutionHandler");
    e.invoke_instance(tpe, "allowCoreThreadTimeOut", "(Z)V", &[Value::Int(1)])
        .expect("allowCoreThreadTimeOut");
    assert!(
        e.invoke_instance(tpe, "getCorePoolSize", "()I", &[])
            .is_err(),
        "the getter is not modelled and must not return a guessed value"
    );
}

#[test]
fn non_jdk_constructor_signature_is_not_invented() {
    let mut e = probe_engine();
    let tpe = e
        .heap
        .alloc_instance(TPE.to_string(), Vec::new())
        .expect("alloc ThreadPoolExecutor");
    let r = e.invoke_instance(tpe, "<init>", "(I)V", &[Value::Int(1)]);
    assert!(r.is_err(), "a signature the JDK does not declare must fail");
}
