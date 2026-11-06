//! `android.view.ScaleGestureDetector`: the AOSP pinch recognizer that
//! `PreviewView` (CameraX) attaches to its touch stream for pinch-to-zoom.
//!
//! Before this module existed, `PreviewView.<init>` reached
//! `new ScaleGestureDetector(Context, OnScaleGestureListener)` and the host
//! answered `NOT_FOUND` ("método ... não resolvido"). That error aborted the
//! frame inside `Recomposer.runFrame`, so the `AndroidView` factory's result
//! was never applied: the Comprix Scanner tab stayed blank. The same class
//! also backs `GestureDetector` siblings in every Compose gesture stack.
//!
//! **Semantics** (AOSP `ScaleGestureDetector`, android-19 and later):
//! - `<init>(Context, OnScaleGestureListener[, Handler])` stores the listener.
//!   Construction has no observable side effect beyond that.
//! - `onTouchEvent(MotionEvent)` returns `true`: AOSP consumes every touch
//!   event it processes and asks for the rest of the stream. The virtual input
//!   pipeline produces single-pointer events only (`MotionEvent.getPointerCount`
//!   is 1), so no two-finger span can form and the listener is never called
//!   for a pinch.
//! - `isInProgress()` is `false`: no scale gesture can begin without a second
//!   pointer.
//!
//! **Limits, documented:** quick scale (double tap followed by a vertical drag,
//! `setQuickScaleEnabled`), stylus scaling, span/focus/scale-factor getters and
//! the `onScale*` callbacks are not modelled. Those calls are not answered with
//! invented values; they fall through to the typed `NOT_FOUND` of an unmodelled
//! method, so a guest that depends on them fails loudly instead of silently.
//!
//! **State** lives in heap fields of the detector object (`rd_*`), like the
//! other gesture hosts, so the GC traces the listener without a host table.

use crate::engine::Engine;
use crate::err::VmExit;
use crate::heap::ObjRef;
use crate::value::Value;

pub(crate) const SCALE_GESTURE_DETECTOR: &str = "Landroid/view/ScaleGestureDetector;";

fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

/// `ScaleGestureDetector.<init>(...)`: stores the listener, no gesture yet.
pub(crate) fn init(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<Option<Value>, VmExit> {
    let listener = match args.get(1) {
        Some(Value::Obj(r)) => *r,
        _ => return Ok(None),
    };
    put(vm, recv, "rd_listener", Value::Obj(listener))?;
    put(vm, recv, "rd_in_progress", Value::Int(0))?;
    Ok(Some(Value::Null))
}

/// Dispatches the modelled `ScaleGestureDetector` instance methods. Returns
/// `Ok(None)` for anything else so the caller reports the typed `NOT_FOUND`.
pub(crate) fn instance_call(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        (
            "<init>",
            "(Landroid/content/Context;Landroid/view/ScaleGestureDetector$OnScaleGestureListener;)V",
        )
        | (
            "<init>",
            "(Landroid/content/Context;Landroid/view/ScaleGestureDetector$OnScaleGestureListener;Landroid/os/Handler;)V",
        ) => return init(vm, recv, args),
        ("onTouchEvent", "(Landroid/view/MotionEvent;)Z") => match args.first() {
            // AOSP returns true for every event it processes.
            Some(Value::Obj(_)) => Value::Int(1),
            _ => Value::Int(0),
        },
        ("isInProgress", "()Z") => Value::Int(0),
        _ => return Ok(None),
    };
    Ok(Some(v))
}
