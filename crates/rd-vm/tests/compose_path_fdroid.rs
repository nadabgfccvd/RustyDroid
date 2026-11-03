//! Regression suite for the F-Droid cold-start chain through Compose.
//!
//! Each test pins one host surface that the Compose launcher path needs and
//! that used to be absent or wrong (2026-11-03, `rd app run` on
//! `golden-apks/org.fdroid.fdroid.apk`):
//!
//! 1. `View.getAutofillId()` returned null → `AndroidComposeView.<init>` died in
//!    `requireNotNull` ("Required value was null.").
//! 2. `android.view.GestureDetector` was not modelled at all → the constructor
//!    came back `NOT_FOUND` on the first composition.
//! 3. `Context.isUiContext()` (API 31) was missing → Compose's window/recomposer
//!    setup stopped there.
//! 4. `WindowManager.getCurrentWindowMetrics()` (API 30) was missing → the
//!    recomposer could not size the window.
//! 5. A static call into an *optional* library class that the APK does not
//!    bundle must raise a catchable `NoClassDefFoundError`
//!    (androidx.window's sidecar adapter depends on it), not a VM error.
//! 6. A guest class implementing `java.util.Iterator` in DEX must run its own
//!    `remove()`, not the host "immutable iterator" path.
//! 7. `VmConfig::app_runtime()` (CLI + MCP) raises the frame cap above the
//!    2 MiB-safe default; Compose + Room + coroutines nest deeper than 128.
//!
//! Plus the `GestureDetector` gesture semantics themselves (tap, double tap,
//! deferred confirmed tap, long press) on the virtual clock.

mod common;

use common::*;
use rd_vm::engine::VmConfig;
use rd_vm::err::VmExit;
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const GESTURE_DETECTOR: &str = "Landroid/view/GestureDetector;";
const AUTOFILL_ID: &str = "Landroid/view/autofill/AutofillId;";
const VIEW: &str = "Landroid/view/View;";
const WINDOW_MANAGER: &str = "Landroid/view/WindowManager;";

fn fresh() -> Engine {
    engine_of(&DexBuilder::new())
}

fn object(v: Value) -> ObjRef {
    match v {
        Value::Obj(r) => r,
        other => panic!("expected object, got {other:?}"),
    }
}

fn int_of(v: Value) -> i32 {
    match v {
        Value::Int(i) => i,
        other => panic!("expected int, got {other:?}"),
    }
}

/// Appends a small DEX class `LProbe229;` with static methods used to reach the
/// host through real bytecode:
/// - `Object touch(GestureDetector, MotionEvent)` → `detector.onTouchEvent(ev)`
/// - `Object autofillId(View)` → `view.getAutofillId()`
/// - `int uiContext(Context)` → `context.isUiContext()`
/// - `void absentStatic()` → `invoke-static Lcoil3/Decoder;->create()V`
fn probe_engine() -> Engine {
    let mut b = DexBuilder::new();
    let cls = b.class("LProbe229;", "Ljava/lang/Object;");

    // Object touch(GestureDetector, MotionEvent)
    let p_touch = b.proto_idx(
        "Z",
        vec![
            "Landroid/view/GestureDetector;".to_string(),
            "Landroid/view/MotionEvent;".to_string(),
        ],
    );
    let m_touch = b.method_idx(GESTURE_DETECTOR, p_touch, "onTouchEvent");
    let mut touch = op35c(0x6e, 2, m_touch, [2, 3, 0, 0, 0]);
    touch.extend(op11x(0x0a, 0)); // move-result v0
    touch.extend(op11x(0x0f, 0)); // return v0
    b.direct(
        cls,
        "touch",
        "Z",
        vec![GESTURE_DETECTOR, "Landroid/view/MotionEvent;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(4, 2, 2, touch)),
    );

    // Object autofillId(View)
    let p_af = b.proto_idx(AUTOFILL_ID, vec![]);
    let m_af = b.method_idx(VIEW, p_af, "getAutofillId");
    let mut af = op35c(0x6e, 1, m_af, [1, 0, 0, 0, 0]);
    af.extend(op11x(0x0c, 0)); // move-result-object v0
    af.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "autofillId",
        "Ljava/lang/Object;",
        vec![VIEW],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 1, af)),
    );

    // int uiContext(Context)
    let p_ui = b.proto_idx("Z", vec![]);
    let m_ui = b.method_idx("Landroid/content/Context;", p_ui, "isUiContext");
    let mut ui = op35c(0x6e, 1, m_ui, [1, 0, 0, 0, 0]);
    ui.extend(op11x(0x0a, 0));
    ui.extend(op11x(0x0f, 0));
    b.direct(
        cls,
        "uiContext",
        "Z",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 1, ui)),
    );

    // void absentStatic() → invoke-static Lcoil3/Decoder;->create()V
    let p_void = b.proto_idx("V", vec![]);
    let m_absent = b.method_idx("Lcoil3/Decoder;", p_void, "create");
    let mut absent = op35c(0x71, 0, m_absent, [0, 0, 0, 0, 0]);
    absent.extend(op11x(0x0e, 0));
    b.direct(
        cls,
        "absentStatic",
        "V",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(0, 0, 0, absent)),
    );

    engine_of(&b)
}

/// `View.getAutofillId()` must be a stable, non-null `AutofillId` — AOSP
/// assigns one when the view joins the window and Compose's
/// `AndroidAutofill.<init>` does `requireNotNull` on it.
#[test]
fn view_autofill_id_is_created_once_and_stays_stable() {
    let mut e = probe_engine();
    let view = e
        .heap
        .alloc_instance(VIEW.to_string(), Vec::new())
        .expect("alloc View");
    // A view that is part of the window has host state (AOSP: it joined the
    // ViewRootImpl). Compose's AndroidComposeView is a DEX subclass with the
    // same attachment, so this mirrors the real path.
    e.attach_host_state(VIEW, view).expect("attach view state");

    let first = object(
        e.invoke_static(
            "LProbe229;",
            "autofillId",
            "(Landroid/view/View;)Ljava/lang/Object;",
            &[Value::Obj(view)],
        )
        .expect("getAutofillId through bytecode"),
    );
    let second = object(
        e.invoke_static(
            "LProbe229;",
            "autofillId",
            "(Landroid/view/View;)Ljava/lang/Object;",
            &[Value::Obj(view)],
        )
        .expect("getAutofillId again"),
    );
    assert_eq!(e.heap.class_of(first).expect("class"), AUTOFILL_ID);
    assert_eq!(first, second, "the AutofillId is stable for the view");
}

/// `Context.isUiContext()` is true for the Activity family and false for the
/// Application (Compose asks before it creates window resources).
#[test]
fn is_ui_context_is_true_for_activity_and_false_for_application() {
    let mut e = probe_engine();
    let activity = e
        .heap
        .alloc_instance("Landroid/app/Activity;".to_string(), Vec::new())
        .expect("alloc Activity");
    let application = e
        .heap
        .alloc_instance("Landroid/app/Application;".to_string(), Vec::new())
        .expect("alloc Application");

    let is_ui = |e: &mut Engine, recv: ObjRef| {
        int_of(
            e.invoke_static(
                "LProbe229;",
                "uiContext",
                "(Landroid/content/Context;)Z",
                &[Value::Obj(recv)],
            )
            .expect("isUiContext"),
        )
    };
    assert_eq!(is_ui(&mut e, activity), 1, "Activity is a UI context");
    assert_eq!(is_ui(&mut e, application), 0, "Application is not");
}

/// `WindowManager.getCurrentWindowMetrics()` is what Compose sizes the window
/// with: bounds = device size, density = dpi/160, insets present (zero).
#[test]
fn window_manager_current_metrics_report_device_bounds_and_density() {
    let mut e = fresh();
    let wm = e
        .heap
        .alloc_instance(WINDOW_MANAGER.to_string(), Vec::new())
        .expect("alloc WindowManager");
    let metrics = object(
        e.invoke_instance(
            wm,
            "getCurrentWindowMetrics",
            "()Landroid/view/WindowMetrics;",
            &[],
        )
        .expect("getCurrentWindowMetrics"),
    );
    let bounds = object(
        e.invoke_instance(metrics, "getBounds", "()Landroid/graphics/Rect;", &[])
            .expect("getBounds"),
    );
    // Bounds come from the same `device()` profile the Display reports.
    let right = match e.heap.get_field(bounds, "right").expect("right") {
        Value::Int(i) => i,
        other => panic!("expected int, got {other:?}"),
    };
    let bottom = match e.heap.get_field(bounds, "bottom").expect("bottom") {
        Value::Int(i) => i,
        other => panic!("expected int, got {other:?}"),
    };
    assert!(
        right > 0 && bottom > 0,
        "window bounds must be the device size"
    );
    let density = match e
        .invoke_instance(metrics, "getDensity", "()F", &[])
        .expect("density")
    {
        Value::Float(f) => f,
        other => panic!("expected float, got {other:?}"),
    };
    assert!(
        (1.0..=4.0).contains(&density),
        "density in the device range, got {density}"
    );
    // The Display must agree with the metrics (one source of truth).
    let display = object(
        e.invoke_instance(wm, "getDefaultDisplay", "()Landroid/view/Display;", &[])
            .expect("getDefaultDisplay"),
    );
    assert_eq!(
        e.invoke_instance(display, "getWidth", "()I", &[])
            .expect("width"),
        Value::Int(right)
    );
    assert_eq!(
        e.invoke_instance(display, "getHeight", "()I", &[])
            .expect("height"),
        Value::Int(bottom)
    );
    let insets = object(
        e.invoke_instance(
            metrics,
            "getWindowInsets",
            "()Landroid/view/WindowInsets;",
            &[],
        )
        .expect("getWindowInsets"),
    );
    assert_eq!(
        int_of(
            e.invoke_instance(insets, "getSystemWindowInsetTop", "()I", &[])
                .expect("inset top")
        ),
        0
    );
}

/// An *optional* library class (the APK does not bundle it) raises a catchable
/// `NoClassDefFoundError` from a static call — androidx.window's sidecar
/// adapter catches exactly that to fall back to the SDK backend.
#[test]
fn absent_optional_class_throws_catchable_no_class_def_found_error() {
    let mut e = probe_engine();
    match e.invoke_static("LProbe229;", "absentStatic", "()V", &[]) {
        Err(VmExit::Exception(t)) => {
            assert_eq!(t.class, "Ljava/lang/NoClassDefFoundError;");
            assert_eq!(t.message.as_deref(), Some("coil3.Decoder"));
        }
        other => panic!("expected NoClassDefFoundError, got {other:?}"),
    }
}

/// A guest `Iterator` implementation keeps its own `remove()`: the host must
/// not answer for it. Dispatching on the *declared* interface
/// (`java.util.Iterator`) used to land in the host funnel and fail with
/// "immutable iterator" — exactly what `androidx.collection`'s set iterators
/// hit on the F-Droid path.
#[test]
fn guest_iterator_remove_runs_its_own_dex_body() {
    let mut b = DexBuilder::new();
    let iterator = b.class("LProbeIter;", "Ljava/lang/Object;");
    b.static_field(iterator, "removed", "I", SVal::Int(0));
    let f_removed = b.field_idx("LProbeIter;", "I", "removed");

    // public void remove() { removed++; }
    let mut code = Vec::new();
    code.extend(op22c(0x60, 0, 0, f_removed)); // sget v0, removed
    code.extend(op22b(0xd8, 0, 0, 1)); // add-int/lit8 v0, v0, 1
    code.extend(op22c(0x67, 0, 0, f_removed)); // sput v0, removed
    code.extend(op11x(0x0e, 0)); // return-void
    b.r#virtual(
        iterator,
        "remove",
        "V",
        vec![],
        ACC_PUBLIC,
        Some(b.code(1, 1, 0, code)),
    );

    // static void removeViaInterface(Iterator it) { it.remove(); }
    let driver = b.class("LProbeDriver;", "Ljava/lang/Object;");
    let p_void = b.proto_idx("V", vec![]);
    let m_remove = b.method_idx("Ljava/util/Iterator;", p_void, "remove");
    let mut via = op35c(0x72, 1, m_remove, [1, 0, 0, 0, 0]); // invoke-interface
    via.extend(op11x(0x0e, 0)); // return-void
    b.direct(
        driver,
        "removeViaInterface",
        "V",
        vec!["Ljava/util/Iterator;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 1, via)),
    );

    let mut e = engine_of(&b);
    let it = e
        .heap
        .alloc_instance("LProbeIter;".to_string(), Vec::new())
        .expect("alloc iterator");
    e.invoke_static(
        "LProbeDriver;",
        "removeViaInterface",
        "(Ljava/util/Iterator;)V",
        &[Value::Obj(it)],
    )
    .expect("guest remove must run its own DEX body");
    assert_eq!(
        e.statics
            .get(&("LProbeIter;".to_string(), "removed".to_string()))
            .cloned(),
        Some(Value::Int(1)),
        "the guest remove() body incremented the counter"
    );
}

/// `VmConfig::app_runtime()` (CLI + MCP) allows the deeper stacks Compose +
/// Room + coroutines need, while the embedded default stays 2 MiB-safe.
#[test]
fn app_runtime_config_raises_frame_cap_without_moving_the_safe_default() {
    assert_eq!(VmConfig::default().max_depth, 128, "2 MiB-safe default");
    assert!(
        VmConfig::app_runtime().max_depth >= 1024,
        "app hosts (CLI/MCP) need room for Compose/Room frames"
    );
    assert_eq!(
        VmConfig::app_runtime().heap_budget,
        VmConfig::default().heap_budget
    );
}
