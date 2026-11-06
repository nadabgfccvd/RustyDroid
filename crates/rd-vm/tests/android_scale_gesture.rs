//! `android.view.ScaleGestureDetector` on the CameraX `PreviewView` path.
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): `PreviewView.<init>`
//! built `new ScaleGestureDetector(Context, OnScaleGestureListener)`, the host
//! had no such class and answered `NOT_FOUND`. That error unwound the
//! `Recomposer` frame before `applyChanges`, so the camera preview never
//! attached. These tests pin the modelled surface through real bytecode and
//! check that the unmodelled surface still fails with a typed error instead of
//! inventing values.

mod common;

use common::*;
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const SGD: &str = "Landroid/view/ScaleGestureDetector;";
const LISTENER: &str = "Landroid/view/ScaleGestureDetector$OnScaleGestureListener;";
const CONTEXT: &str = "Landroid/content/Context;";
const MOTION: &str = "Landroid/view/MotionEvent;";

fn object(v: Value) -> ObjRef {
    match v {
        Value::Obj(r) => r,
        other => panic!("expected object, got {other:?}"),
    }
}

/// Appends `LScaleProbe;` with static methods that reach the host through
/// real bytecode:
/// - `Object make(Context, OnScaleGestureListener)` → `new ScaleGestureDetector(ctx, l)`
/// - `boolean touch(ScaleGestureDetector, MotionEvent)` → `onTouchEvent(ev)`
/// - `boolean inProgress(ScaleGestureDetector)` → `isInProgress()`
/// - `float factor(ScaleGestureDetector)` → `getScaleFactor()` (not modelled)
fn probe_engine() -> Engine {
    let mut b = DexBuilder::new();
    let cls = b.class("LScaleProbe;", "Ljava/lang/Object;");
    let sgd_t = b.type_idx(SGD);

    // Object make(Context, OnScaleGestureListener): v0 = new SGD; <init>(v1, v2); return v0
    let p_init = b.proto_idx("V", vec![CONTEXT.to_string(), LISTENER.to_string()]);
    let m_init = b.method_idx(SGD, p_init, "<init>");
    let mut make = op21c(0x22, 0, sgd_t); // new-instance v0, SGD
    make.extend(op35c(0x70, 3, m_init, [0, 1, 2, 0, 0])); // invoke-direct {v0,v1,v2}, <init>
    make.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "make",
        "Ljava/lang/Object;",
        vec![CONTEXT, LISTENER],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(3, 2, 3, make)),
    );

    // boolean touch(SGD, MotionEvent): onTouchEvent(p1)
    let p_touch = b.proto_idx("Z", vec![MOTION.to_string()]);
    let m_touch = b.method_idx(SGD, p_touch, "onTouchEvent");
    let mut touch = op35c(0x6e, 2, m_touch, [2, 3, 0, 0, 0]);
    touch.extend(op11x(0x0a, 0)); // move-result v0
    touch.extend(op11x(0x0f, 0)); // return v0
    b.direct(
        cls,
        "touch",
        "Z",
        vec![SGD, MOTION],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 2, 2, touch)),
    );

    // boolean inProgress(SGD): isInProgress()
    let p_ip = b.proto_idx("Z", vec![]);
    let m_ip = b.method_idx(SGD, p_ip, "isInProgress");
    let mut ip = op35c(0x6e, 1, m_ip, [2, 0, 0, 0, 0]);
    ip.extend(op11x(0x0a, 0));
    ip.extend(op11x(0x0f, 0));
    b.direct(
        cls,
        "inProgress",
        "Z",
        vec![SGD],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(3, 1, 1, ip)),
    );

    // float factor(SGD): getScaleFactor() — deliberately unmodelled
    let p_f = b.proto_idx("F", vec![]);
    let m_f = b.method_idx(SGD, p_f, "getScaleFactor");
    let mut fac = op35c(0x6e, 1, m_f, [2, 0, 0, 0, 0]);
    fac.extend(op11x(0x0a, 0));
    fac.extend(op11x(0x0f, 0));
    b.direct(
        cls,
        "factor",
        "F",
        vec![SGD],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(3, 1, 1, fac)),
    );

    engine_of(&b)
}

/// `new ScaleGestureDetector(context, listener)` must construct — this is the
/// call `PreviewView.<init>` makes on the Scanner tab.
#[test]
fn constructor_builds_detector_for_preview_view() {
    let mut e = probe_engine();
    let listener = e
        .heap
        .alloc_instance(LISTENER.to_string(), Vec::new())
        .expect("alloc listener");
    let det = object(
        e.invoke_static(
            "LScaleProbe;",
            "make",
            "(Landroid/content/Context;Landroid/view/ScaleGestureDetector$OnScaleGestureListener;)Ljava/lang/Object;",
            &[Value::Null, Value::Obj(listener)],
        )
        .expect("new ScaleGestureDetector through bytecode"),
    );
    assert_eq!(e.heap.class_of(det).expect("class"), SGD);
    let in_progress = e
        .invoke_static(
            "LScaleProbe;",
            "inProgress",
            "(Landroid/view/ScaleGestureDetector;)Z",
            &[Value::Obj(det)],
        )
        .expect("isInProgress");
    assert_eq!(
        in_progress,
        Value::Int(0),
        "no pinch can start without two pointers"
    );
}

/// AOSP consumes the single-pointer events the virtual input produces.
#[test]
fn on_touch_event_is_consumed_like_aosp() {
    let mut e = probe_engine();
    let listener = e
        .heap
        .alloc_instance(LISTENER.to_string(), Vec::new())
        .expect("alloc listener");
    let det = object(
        e.invoke_static(
            "LScaleProbe;",
            "make",
            "(Landroid/content/Context;Landroid/view/ScaleGestureDetector$OnScaleGestureListener;)Ljava/lang/Object;",
            &[Value::Null, Value::Obj(listener)],
        )
        .expect("construct"),
    );
    let ev = e
        .heap
        .alloc_instance(MOTION.to_string(), Vec::new())
        .expect("alloc MotionEvent");
    let consumed = e
        .invoke_static(
            "LScaleProbe;",
            "touch",
            "(Landroid/view/ScaleGestureDetector;Landroid/view/MotionEvent;)Z",
            &[Value::Obj(det), Value::Obj(ev)],
        )
        .expect("onTouchEvent through bytecode");
    assert_eq!(consumed, Value::Int(1));
}

/// Methods outside the modelled surface stay typed errors (no invented
/// scale factor): the guest sees the failure instead of a silent zero.
#[test]
fn unmodelled_scale_factor_fails_with_typed_error() {
    let mut e = probe_engine();
    let listener = e
        .heap
        .alloc_instance(LISTENER.to_string(), Vec::new())
        .expect("alloc listener");
    let det = object(
        e.invoke_static(
            "LScaleProbe;",
            "make",
            "(Landroid/content/Context;Landroid/view/ScaleGestureDetector$OnScaleGestureListener;)Ljava/lang/Object;",
            &[Value::Null, Value::Obj(listener)],
        )
        .expect("construct"),
    );
    let r = e.invoke_static(
        "LScaleProbe;",
        "factor",
        "(Landroid/view/ScaleGestureDetector;)F",
        &[Value::Obj(det)],
    );
    assert!(
        r.is_err(),
        "getScaleFactor is not modelled and must not return a guess"
    );
}
