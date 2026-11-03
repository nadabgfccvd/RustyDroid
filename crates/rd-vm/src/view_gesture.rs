//! `android.view.GestureDetector`: the AOSP gesture recognizer apps attach to a
//! `View` (`view.setOnTouchListener`/`onTouchEvent` → `detector.onTouchEvent`).
//!
//! It is the second host surface Compose needs on the launcher path: after
//! `View.getAutofillId()` was modelled, the F-Droid cold start stopped at
//! `GestureDetector.<init>(Context, OnGestureListener)` — a class no host
//! module owned, so the constructor came back `NOT_FOUND`.
//!
//! **Semantics** follow AOSP `GestureDetector` with the deterministic virtual
//! clock (`Engine::clock_ms`, the same clock `postDelayed` uses):
//! - **Tap:** `ACTION_UP` inside touch slop and before the long-press timeout
//!   → `onSingleTapUp`. `onSingleTapConfirmed` is deferred by
//!   `DOUBLE_TAP_TIMEOUT` (300 ms) through [`pending`]-style delivery on
//!   `advance_clock`/`drain_looper`, so a second tap in the window converts it
//!   into `onDoubleTap` + `onDoubleTapEvent` instead (AOSP behaviour).
//! - **Double tap:** `ACTION_DOWN` within `DOUBLE_TAP_TIMEOUT` and touch slop
//!   of the last tap → `onDoubleTap` on the down, `onDoubleTapEvent` on the
//!   following move/up.
//! - **Long press:** pointer still down, inside slop, after
//!   `ViewConfiguration.getLongPressTimeout()` (500 ms) → `onLongPress`
//!   (delivered by the same clock drain while the pointer is still down).
//! - **Scroll/fling:** movement beyond slop → `onScroll(down, ev, dx, dy)`;
//!   `ACTION_UP` with an estimated velocity above
//!   `ViewConfiguration.getScaledMinimumFlingVelocity()` → `onFling(down, ev,
//!   vx, vy)`. Velocity is the displacement over the gesture duration (there is
//!   no sensor noise to smooth).
//!
//! **Limits, documented:** `setContextClickListener`/`onContextClick` are not
//! modelled (no right-click/stylus button in the virtual input), and pointer
//! velocity uses the mean speed rather than `VelocityTracker`'s least-squares
//! fit.
//!
//! **State** lives in heap fields of the detector object (`rd_*`), so the GC
//! traces the listeners without a host-side table.

use crate::engine::Engine;
use crate::err::VmExit;
use crate::heap::ObjRef;
use crate::value::Value;

pub(crate) const GESTURE_DETECTOR: &str = "Landroid/view/GestureDetector;";

/// AOSP `GestureDetector.DOUBLE_TAP_TIMEOUT`.
const DOUBLE_TAP_TIMEOUT: u64 = 300;
/// AOSP `ViewConfiguration.getLongPressTimeout()` default.
const LONG_PRESS_TIMEOUT: u64 = 500;
/// AOSP `ViewConfiguration.getScaledMinimumFlingVelocity()` at density 1
/// (`MINIMUM_FLING_VELOCITY` = 50 dp/s); the host scales by density.
const MIN_FLING_VELOCITY_DP: f32 = 50.0;
/// AOSP `ViewConfiguration.getScaledTouchSlop()` at density 1 (`8 dp`).
const TOUCH_SLOP_DP: f32 = 8.0;

fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

fn get(vm: &Engine, r: ObjRef, f: &str) -> Value {
    vm.heap.get_field(r, f).unwrap_or(Value::Null)
}

fn getf(vm: &Engine, r: ObjRef, f: &str) -> f32 {
    match get(vm, r, f) {
        Value::Float(v) => v,
        Value::Int(i) => i as f32,
        Value::Double(d) => d as f32,
        _ => 0.0,
    }
}

fn getl(vm: &Engine, r: ObjRef, f: &str) -> i64 {
    match get(vm, r, f) {
        Value::Long(l) => l,
        Value::Int(i) => i as i64,
        _ => 0,
    }
}

fn geti(vm: &Engine, r: ObjRef, f: &str) -> i32 {
    match get(vm, r, f) {
        Value::Int(i) => i,
        Value::Long(l) => l as i32,
        _ => 0,
    }
}

fn obj_of(vm: &Engine, r: ObjRef, f: &str) -> Option<ObjRef> {
    match get(vm, r, f) {
        Value::Obj(o) if o != 0 => Some(o),
        _ => None,
    }
}

/// Density-scaled AOSP constant (`ViewConfiguration` density, `RD_DENSITY_DPI`).
fn scaled(_vm: &Engine, dp: f32) -> f32 {
    dp * crate::view_layout::density()
}

fn clock(vm: &Engine) -> u64 {
    vm.fw.clock_ms
}

/// Calls `listener.<name><sig>` on guest code; a listener that does not
/// declare the method gets AOSP's default (the interfaces have no bodies, so
/// this is only reached by `SimpleOnGestureListener`-style base classes whose
/// defaults are no-ops — `Ok(None)` here means "nothing to do").
fn call_listener(
    vm: &mut Engine,
    listener: ObjRef,
    name: &str,
    sig: &str,
    args: Vec<Value>,
) -> Result<(), VmExit> {
    let class = vm.heap.class_of(listener)?.to_string();
    let Some((dex, def, m)) = vm.cp.resolve_method(&class, name, sig) else {
        return Ok(());
    };
    let mut ins = vec![Value::Obj(listener)];
    ins.extend(args);
    vm.call(dex, def, &m, ins)?;
    Ok(())
}

/// `GestureDetector.<init>(...)`: stores the listener and AOSP defaults.
pub(crate) fn init(vm: &mut Engine, recv: ObjRef, args: &[Value]) -> Result<Option<Value>, VmExit> {
    let listener = match args.first() {
        Some(Value::Obj(r)) => *r,
        _ => return Ok(None),
    };
    put(vm, recv, "rd_listener", Value::Obj(listener))?;
    put(vm, recv, "rd_double_tap", Value::Null)?;
    put(vm, recv, "rd_longpress", Value::Int(1))?;
    put(vm, recv, "rd_scrolling", Value::Int(0))?;
    put(vm, recv, "rd_is_double_tap", Value::Int(0))?;
    put(vm, recv, "rd_tap_pending", Value::Int(0))?;
    put(vm, recv, "rd_tap_time", Value::Long(0))?;
    put(vm, recv, "rd_tap_x", Value::Float(0.0))?;
    put(vm, recv, "rd_tap_y", Value::Float(0.0))?;
    Ok(Some(Value::Null))
}

/// Event accessors (the `MotionEvent` host keeps its state in `rd_*` fields).
fn action(vm: &Engine, ev: ObjRef) -> i32 {
    geti(vm, ev, "rd_action")
}

fn event_xy(vm: &Engine, ev: ObjRef) -> (f32, f32) {
    (getf(vm, ev, "rd_x"), getf(vm, ev, "rd_y"))
}

fn event_time(vm: &Engine, ev: ObjRef) -> i64 {
    getl(vm, ev, "rd_time")
}

fn event_down_time(vm: &Engine, ev: ObjRef) -> i64 {
    getl(vm, ev, "rd_down")
}

/// `GestureDetector.onTouchEvent(MotionEvent)`.
pub(crate) fn on_touch_event(
    vm: &mut Engine,
    recv: ObjRef,
    ev: ObjRef,
) -> Result<Option<Value>, VmExit> {
    let Some(listener) = obj_of(vm, recv, "rd_listener") else {
        return Ok(None);
    };
    let act = action(vm, ev);
    let (x, y) = event_xy(vm, ev);
    let time = event_time(vm, ev);
    let slop = scaled(vm, TOUCH_SLOP_DP);
    let double_tap_listener = obj_of(vm, recv, "rd_double_tap");
    match act {
        crate::android_motion::ACTION_DOWN => {
            let last_tap = getl(vm, recv, "rd_tap_time") as u64;
            let pending = geti(vm, recv, "rd_tap_pending") != 0;
            let near = (x - getf(vm, recv, "rd_tap_x")).abs() <= slop
                && (y - getf(vm, recv, "rd_tap_y")).abs() <= slop;
            // A second down inside the double-tap window turns the pending
            // single tap into a double tap (AOSP cancels the deferred
            // `onSingleTapConfirmed`).
            if pending && near && (time as u64).saturating_sub(last_tap) <= DOUBLE_TAP_TIMEOUT {
                put(vm, recv, "rd_tap_pending", Value::Int(0))?;
                put(vm, recv, "rd_is_double_tap", Value::Int(1))?;
                if let Some(dt) = double_tap_listener {
                    call_listener(
                        vm,
                        dt,
                        "onDoubleTap",
                        "(Landroid/view/MotionEvent;)Z",
                        vec![Value::Obj(ev)],
                    )?;
                }
            } else {
                put(vm, recv, "rd_is_double_tap", Value::Int(0))?;
            }
            put(vm, recv, "rd_down_x", Value::Float(x))?;
            put(vm, recv, "rd_down_y", Value::Float(y))?;
            put(vm, recv, "rd_down_time", Value::Long(time))?;
            put(vm, recv, "rd_last_x", Value::Float(x))?;
            put(vm, recv, "rd_last_y", Value::Float(y))?;
            put(vm, recv, "rd_scrolling", Value::Int(0))?;
            put(vm, recv, "rd_longpress_fired", Value::Int(0))?;
            put(vm, recv, "rd_press_time", Value::Long(time))?;
            watch(vm, recv);
            Ok(Some(Value::Int(1)))
        }
        crate::android_motion::ACTION_MOVE => {
            let down_x = getf(vm, recv, "rd_down_x");
            let down_y = getf(vm, recv, "rd_down_y");
            let last_x = getf(vm, recv, "rd_last_x");
            let last_y = getf(vm, recv, "rd_last_y");
            let mut scrolling = geti(vm, recv, "rd_scrolling") != 0;
            if !scrolling
                && ((x - down_x).abs() > slop || (y - down_y).abs() > slop)
                && geti(vm, recv, "rd_longpress_fired") == 0
            {
                scrolling = true;
                put(vm, recv, "rd_scrolling", Value::Int(1))?;
            }
            if scrolling {
                let dx = last_x - x;
                let dy = last_y - y;
                if dx != 0.0 || dy != 0.0 {
                    call_listener(
                        vm,
                        listener,
                        "onScroll",
                        "(Landroid/view/MotionEvent;Landroid/view/MotionEvent;FF)Z",
                        vec![
                            Value::Null,
                            Value::Obj(ev),
                            Value::Float(dx),
                            Value::Float(dy),
                        ],
                    )?;
                }
            }
            if geti(vm, recv, "rd_is_double_tap") != 0 {
                if let Some(dt) = double_tap_listener {
                    call_listener(
                        vm,
                        dt,
                        "onDoubleTapEvent",
                        "(Landroid/view/MotionEvent;)Z",
                        vec![Value::Obj(ev)],
                    )?;
                }
            }
            put(vm, recv, "rd_last_x", Value::Float(x))?;
            put(vm, recv, "rd_last_y", Value::Float(y))?;
            Ok(Some(Value::Int(1)))
        }
        crate::android_motion::ACTION_UP => {
            let down_x = getf(vm, recv, "rd_down_x");
            let down_y = getf(vm, recv, "rd_down_y");
            let down_time = event_down_time(vm, ev).max(getl(vm, recv, "rd_down_time"));
            let scrolling = geti(vm, recv, "rd_scrolling") != 0;
            let within_slop = (x - down_x).abs() <= slop && (y - down_y).abs() <= slop;
            let elapsed = (time - down_time).max(0) as u64;
            let is_double = geti(vm, recv, "rd_is_double_tap") != 0;
            if is_double {
                if let Some(dt) = double_tap_listener {
                    call_listener(
                        vm,
                        dt,
                        "onDoubleTapEvent",
                        "(Landroid/view/MotionEvent;)Z",
                        vec![Value::Obj(ev)],
                    )?;
                }
                put(vm, recv, "rd_is_double_tap", Value::Int(0))?;
                put(vm, recv, "rd_tap_time", Value::Long(0))?;
            } else if within_slop && elapsed < LONG_PRESS_TIMEOUT {
                // `onSingleTapUp` is immediate; `onSingleTapConfirmed` waits
                // out the double-tap window (see `deliver_pending`).
                call_listener(
                    vm,
                    listener,
                    "onSingleTapUp",
                    "(Landroid/view/MotionEvent;)Z",
                    vec![Value::Obj(ev)],
                )?;
                put(vm, recv, "rd_tap_pending", Value::Int(1))?;
                put(vm, recv, "rd_tap_time", Value::Long(time))?;
                put(vm, recv, "rd_tap_x", Value::Float(x))?;
                put(vm, recv, "rd_tap_y", Value::Float(y))?;
                put(vm, recv, "rd_tap_event_x", Value::Float(x))?;
                put(vm, recv, "rd_tap_event_y", Value::Float(y))?;
                watch(vm, recv);
            } else if scrolling {
                let dt = elapsed.max(1) as f32;
                let vx = (x - down_x) / dt * 1000.0;
                let vy = (y - down_y) / dt * 1000.0;
                let min = scaled(vm, MIN_FLING_VELOCITY_DP);
                if (vx * vx + vy * vy).sqrt() >= min {
                    call_listener(
                        vm,
                        listener,
                        "onFling",
                        "(Landroid/view/MotionEvent;Landroid/view/MotionEvent;FF)Z",
                        vec![
                            Value::Null,
                            Value::Obj(ev),
                            Value::Float(vx),
                            Value::Float(vy),
                        ],
                    )?;
                }
            }
            put(vm, recv, "rd_scrolling", Value::Int(0))?;
            put(vm, recv, "rd_longpress_fired", Value::Int(0))?;
            put(vm, recv, "rd_press_time", Value::Long(0))?;
            Ok(Some(Value::Int(1)))
        }
        _ => Ok(Some(Value::Int(0))),
    }
}

/// Instance entry point. `None` keeps the DEX implementation (there is none for
/// a platform class, so every call the app makes lands here).
pub(crate) fn instance_call(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    let v = match (name, sig) {
        ("<init>", "(Landroid/content/Context;Landroid/view/GestureDetector$OnGestureListener;)V")
        | (
            "<init>",
            "(Landroid/content/Context;Landroid/view/GestureDetector$OnGestureListener;Landroid/os/Handler;)V",
        )
        | (
            "<init>",
            "(Landroid/content/Context;Landroid/view/GestureDetector$OnGestureListener;Landroid/os/Handler;Z)V",
        ) => return init(vm, recv, args),
        ("setOnDoubleTapListener", "(Landroid/view/GestureDetector$OnDoubleTapListener;)V") => {
            put(vm, recv, "rd_double_tap", args.first().cloned().unwrap_or(Value::Null))?;
            Value::Null
        }
        ("setIsLongpressEnabled", "(Z)V") => {
            let on = matches!(args.first(), Some(Value::Int(i)) if *i != 0);
            put(vm, recv, "rd_longpress", Value::Int(i32::from(on)))?;
            Value::Null
        }
        ("isLongpressEnabled", "()Z") => Value::Int(i32::from(geti(vm, recv, "rd_longpress") != 0)),
        ("onTouchEvent", "(Landroid/view/MotionEvent;)Z") => {
            let Some(Value::Obj(ev)) = args.first().cloned() else {
                return Ok(Some(Value::Int(0)));
            };
            return on_touch_event(vm, recv, ev);
        }
        // No stylus/mouse secondary button and no generic motion in the
        // virtual input pipeline: AOSP returns false when nothing consumed it.
        ("onGenericMotionEvent", "(Landroid/view/MotionEvent;)Z") => Value::Int(0),
        ("setContextClickListener", "(Landroid/view/GestureDetector$OnContextClickListener;)V") => {
            Value::Null
        }
        ("setOnContextClickListener", "(Landroid/view/GestureDetector$OnContextClickListener;)V") => {
            Value::Null
        }
        _ => return Ok(None),
    };
    Ok(Some(v))
}

/// Clock drain: pending single taps whose double-tap window has elapsed and
/// long presses whose 500 ms elapsed while the finger stayed down.
///
/// Called from `drain_looper`/`advance_clock` next to `perms::deliver_pending`,
/// so the callbacks fire in virtual time order with `postDelayed` work.
pub(crate) fn deliver_pending(vm: &mut Engine) -> Result<usize, VmExit> {
    let mut delivered = 0usize;
    let now = clock(vm);
    let detectors = vm.fw.gesture_watch.clone();
    for recv in detectors {
        let Some(listener) = obj_of(vm, recv, "rd_listener") else {
            continue;
        };
        // Long press: still down, inside slop, timeout reached, not fired yet.
        let press_time = getl(vm, recv, "rd_press_time") as u64;
        if press_time != 0
            && geti(vm, recv, "rd_longpress") != 0
            && geti(vm, recv, "rd_longpress_fired") == 0
            && geti(vm, recv, "rd_scrolling") == 0
            && now.saturating_sub(press_time) >= LONG_PRESS_TIMEOUT
        {
            put(vm, recv, "rd_longpress_fired", Value::Int(1))?;
            // AOSP hands `onLongPress` the down event/most recent event; the
            // event object is not kept alive, so the listener gets the down
            // coordinates through a fresh event.
            let ev = crate::android_motion::new_event(
                vm,
                crate::android_motion::ACTION_DOWN,
                getf(vm, recv, "rd_down_x"),
                getf(vm, recv, "rd_down_y"),
                0.0,
                0.0,
                press_time as i64,
                press_time as i64,
            )?;
            call_listener(
                vm,
                listener,
                "onLongPress",
                "(Landroid/view/MotionEvent;)V",
                vec![Value::Obj(ev)],
            )?;
            delivered += 1;
        }
        // Deferred single tap confirmed.
        if geti(vm, recv, "rd_tap_pending") != 0 {
            let tap_time = getl(vm, recv, "rd_tap_time") as u64;
            if now.saturating_sub(tap_time) >= DOUBLE_TAP_TIMEOUT {
                put(vm, recv, "rd_tap_pending", Value::Int(0))?;
                if let Some(dt) = obj_of(vm, recv, "rd_double_tap") {
                    let ev = crate::android_motion::new_event(
                        vm,
                        crate::android_motion::ACTION_UP,
                        getf(vm, recv, "rd_tap_event_x"),
                        getf(vm, recv, "rd_tap_event_y"),
                        0.0,
                        0.0,
                        tap_time as i64,
                        tap_time as i64,
                    )?;
                    call_listener(
                        vm,
                        dt,
                        "onSingleTapConfirmed",
                        "(Landroid/view/MotionEvent;)Z",
                        vec![Value::Obj(ev)],
                    )?;
                    delivered += 1;
                }
            }
        }
        let still_pending = geti(vm, recv, "rd_tap_pending") != 0
            || (geti(vm, recv, "rd_press_time") as i64 != 0
                && geti(vm, recv, "rd_longpress_fired") == 0
                && geti(vm, recv, "rd_scrolling") == 0);
        if !still_pending {
            vm.fw.gesture_watch.retain(|r| *r != recv);
        }
    }
    Ok(delivered)
}

/// Arms the deferred-callback drain for `recv` (idempotent).
fn watch(vm: &mut Engine, recv: ObjRef) {
    if !vm.fw.gesture_watch.contains(&recv) {
        vm.fw.gesture_watch.push(recv);
    }
}
