//! Gaps found by a real casual game on the headless runtime:
//! 1. M3 XML layouts: a DEX `GameView` weighted inside a host LinearLayout
//!    must receive the leftover height (touch/drag reach it).
//! 2. AlertDialog must appear in the post-tap dump.
//! 3. android.media SoundPool/AudioAttributes must resolve (silent, no crash),
//!    and `Class.newInstance()` must work for platform classes.

#[macro_use]
mod common;
use common::*;
use rd_vm::framework::HostObj;
use rd_vm::value::Value;

/// Activity → Window → `content` (the XML root), as `setContentView` leaves it.
fn install_window(e: &mut rd_vm::Engine, content: u32) -> u32 {
    let activity = e
        .heap
        .alloc_instance("Landroid/app/Activity;".into(), vec![])
        .unwrap();
    e.attach_host_state("Landroid/app/Activity;", activity)
        .unwrap();
    let window = e
        .heap
        .alloc_instance("Landroid/view/Window;".into(), vec![])
        .unwrap();
    e.fw.objects.insert(
        window,
        HostObj::Window {
            content: Some(content),
        },
    );
    e.fw.objects.insert(
        activity,
        HostObj::Activity {
            window: Some(window),
            finished: false,
            intent: None,
        },
    );
    e.fw.current_activity = Some(activity);
    activity
}

fn host_view(e: &mut rd_vm::Engine, class: &str) -> u32 {
    let v = e.heap.alloc_instance(class.into(), vec![]).unwrap();
    e.attach_host_state(class, v).unwrap();
    v
}

fn params(e: &mut rd_vm::Engine, width: i32, height: i32, weight: Option<f32>) -> u32 {
    let p = e
        .heap
        .alloc_instance("Landroid/widget/LinearLayout$LayoutParams;".into(), vec![])
        .unwrap();
    e.heap.put_field(p, "width", Value::Int(width)).unwrap();
    e.heap.put_field(p, "height", Value::Int(height)).unwrap();
    e.heap
        .put_field(p, "weight", Value::Float(weight.unwrap_or(0.0)))
        .unwrap();
    p
}

fn add(e: &mut rd_vm::Engine, parent: u32, child: u32, p: u32) {
    e.invoke_instance(
        parent,
        "addView",
        "(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V",
        &[Value::Obj(child), Value::Obj(p)],
    )
    .unwrap();
}

fn bounds(e: &rd_vm::Engine, v: u32) -> (i32, i32, i32, i32) {
    match e.fw.get(v) {
        Some(HostObj::View { x, y, w, h, .. }) => (*x, *y, *w, *h),
        other => panic!("not a view: {other:?}"),
    }
}

#[test]
fn weighted_dex_view_under_host_linear_layout_fills_leftover_height() {
    let mut b = DexBuilder::new();
    b.class("LGameView;", "Landroid/view/View;");
    let mut e = engine_of(&b);

    let root = host_view(&mut e, "Landroid/widget/LinearLayout;");
    e.invoke_instance(root, "setOrientation", "(I)V", &[Value::Int(1)])
        .unwrap();

    // header: wrap-content host TextView (48px touch row)
    let header = host_view(&mut e, "Landroid/widget/TextView;");
    let p = params(&mut e, -1, -2, None);
    add(&mut e, root, header, p);

    // game: DEX view, match-parent width, height 0 + weight 1 → rest of window
    let game = e.heap.alloc_instance("LGameView;".into(), vec![]).unwrap();
    e.attach_host_state("LGameView;", game).unwrap();
    let p = params(&mut e, -1, 0, Some(1.0));
    add(&mut e, root, game, p);

    install_window(&mut e, root);
    e.ensure_laid_out().unwrap();

    let (vw, vh) = e.viewport_px();
    let (_, hy, _, hh) = bounds(&e, header);
    assert_eq!((hy, hh), (0, 48), "header keeps its wrap-content row");
    let (gx, gy, gw, gh) = bounds(&e, game);
    assert_eq!(
        (gx, gy),
        (0, 48),
        "weighted GameView must start below the header, not at the window top"
    );
    assert_eq!(gw, vw);
    assert_eq!(
        gh,
        vh - 48,
        "weighted GameView must take the leftover height"
    );
}

#[test]
fn alert_dialog_is_listed_in_post_tap_dump() {
    let mut e = rd_vm::Engine::new(Vec::new(), rd_vm::VmConfig::default());
    let b = e
        .heap
        .alloc_instance("Landroid/app/AlertDialog$Builder;".into(), vec![])
        .unwrap();
    e.attach_host_state("Landroid/app/AlertDialog$Builder;", b)
        .unwrap();
    let title = Value::Obj(e.heap.alloc_string("Apagar?".into()).unwrap());
    e.invoke_instance(
        b,
        "setTitle",
        "(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;",
        &[title],
    )
    .unwrap();
    let msg = Value::Obj(e.heap.alloc_string("Confirma exclusão?".into()).unwrap());
    e.invoke_instance(
        b,
        "setMessage",
        "(Ljava/lang/CharSequence;)Landroid/app/AlertDialog$Builder;",
        &[msg],
    )
    .unwrap();
    let ok = Value::Obj(e.heap.alloc_string("Sim".into()).unwrap());
    e.invoke_instance(
        b,
        "setPositiveButton",
        "(Ljava/lang/CharSequence;Landroid/content/DialogInterface$OnClickListener;)Landroid/app/AlertDialog$Builder;",
        &[ok, Value::Null],
    )
    .unwrap();
    e.invoke_instance(b, "show", "()Landroid/app/AlertDialog;", &[])
        .unwrap();

    let dump = e.dump_ui();
    assert!(
        dump.contains("--- dialog window"),
        "dialog window missing from dump: {dump}"
    );
    assert!(dump.contains("text=\"Apagar?\""), "title missing: {dump}");
    assert!(
        dump.contains("text=\"Confirma exclusão?\""),
        "message missing: {dump}"
    );
    assert!(
        dump.contains("Button") && dump.contains("text=\"Sim\""),
        "dialog button missing: {dump}"
    );
}

#[test]
fn class_new_instance_works_for_platform_classes() {
    let mut e = rd_vm::Engine::new(Vec::new(), rd_vm::VmConfig::default());
    let name = Value::Obj(
        e.heap
            .alloc_string("android.media.AudioAttributes$Builder".into())
            .unwrap(),
    );
    let class = e
        .invoke_static(
            "Ljava/lang/Class;",
            "forName",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[name],
        )
        .unwrap();
    let Value::Obj(class) = class else {
        panic!("forName returned {class:?}");
    };
    let builder = e
        .invoke_instance(class, "newInstance", "()Ljava/lang/Object;", &[])
        .unwrap();
    let Value::Obj(builder) = builder else {
        panic!("newInstance returned {builder:?}");
    };
    e.invoke_instance(
        builder,
        "setUsage",
        "(I)Landroid/media/AudioAttributes$Builder;",
        &[Value::Int(14)],
    )
    .unwrap();
    let attrs = e
        .invoke_instance(builder, "build", "()Landroid/media/AudioAttributes;", &[])
        .unwrap();
    let Value::Obj(attrs) = attrs else {
        panic!("build returned {attrs:?}");
    };
    assert_eq!(
        e.invoke_instance(attrs, "getUsage", "()I", &[]).unwrap(),
        Value::Int(14)
    );
}

#[test]
fn sound_pool_is_silent_but_consistent() {
    let mut e = rd_vm::Engine::new(Vec::new(), rd_vm::VmConfig::default());
    let pool = e
        .heap
        .alloc_instance("Landroid/media/SoundPool;".into(), vec![])
        .unwrap();
    e.attach_host_state("Landroid/media/SoundPool;", pool)
        .unwrap();
    e.invoke_instance(
        pool,
        "<init>",
        "(III)V",
        &[Value::Int(4), Value::Int(3), Value::Int(0)],
    )
    .unwrap();
    let a = e
        .invoke_instance(
            pool,
            "load",
            "(Landroid/content/Context;II)I",
            &[Value::Null, Value::Int(1), Value::Int(0)],
        )
        .unwrap();
    let b = e
        .invoke_instance(
            pool,
            "load",
            "(Landroid/content/Context;II)I",
            &[Value::Null, Value::Int(2), Value::Int(0)],
        )
        .unwrap();
    assert_ne!(a, Value::Int(0));
    assert_ne!(a, b, "each load gets its own sound id");
    let stream = e
        .invoke_instance(
            pool,
            "play",
            "(IFFIIF)I",
            &[
                a,
                Value::Float(1.0),
                Value::Float(1.0),
                Value::Int(0),
                Value::Int(0),
                Value::Float(1.0),
            ],
        )
        .unwrap();
    assert_ne!(stream, Value::Int(0), "play accepts the request");
    e.invoke_instance(pool, "release", "()V", &[]).unwrap();
}
