//! `android:background="?attr/x"` is resolved through the activity's theme
//! (manifest `application android:theme` → style bag → attribute), not left
//! as a transparent stand-in. Covers: a theme item that references a color
//! resource, an inline color item, an attribute the theme does not define
//! (transparent stand-in), and no theme at all (transparent stand-in).

#[macro_use]
mod common;

use common::apkfix::{
    a, build_apk, build_apk_themed, build_axml, ArscFix, ArscVal, AxNode, AxVal, NO_INDEX,
};
use common::*;
use rd_vm::value::Value;

const ACTIVITY: &str = "LThemeMain;";
const PACKAGE: &str = "com.test.theme";

fn register_platform_classes(b: &mut DexBuilder) {
    b.class("Landroid/view/View;", "Ljava/lang/Object;");
    b.class("Landroid/view/ViewGroup;", "Landroid/view/View;");
    b.class("Landroid/os/Bundle;", "Ljava/lang/Object;");
    b.class("Landroid/content/Intent;", "Ljava/lang/Object;");
}

fn op31i(op: u8, reg: u8, lit: i32) -> Vec<u16> {
    vec![
        (op as u16) | ((reg as u16) << 8),
        (lit & 0xFFFF) as u16,
        ((lit >> 16) & 0xFFFF) as u16,
    ]
}

/// Activity whose `onCreate` calls `setContentView(layout)`.
fn engine_with_layout(apk_bytes: Vec<u8>, layout: u32) -> rd_vm::Engine {
    let mut b = DexBuilder::new();
    register_platform_classes(&mut b);
    let p_iv = b.proto_idx("V", vec!["I".to_string()]);
    let set_content_i = b.method_idx("Landroid/app/Activity;", p_iv, "setContentView");
    let main = b.class(ACTIVITY, "Landroid/app/Activity;");
    b.direct(
        main,
        "onCreate",
        "V",
        vec!["Landroid/os/Bundle;"],
        ACC_PUBLIC,
        Some(b.code(4, 2, 2, {
            let mut u = op31i(0x14, 1, layout as i32);
            u.extend(op35c(0x6E, 2, set_content_i, [2, 1, 0, 0, 0]));
            u.extend(op10x(0x0E));
            u
        })),
    );
    let mut e = engine_of(&b);
    e.set_resources(rd_apk::Apk::from_bytes(apk_bytes).expect("synthetic APK parses"));
    e
}

struct Fixture {
    arsc: ArscFix,
    layout: u32,
    root_id: u32,
    attr_bg: u32,
    style_ref: u32,
    style_inline: u32,
    style_empty: u32,
}

fn fixture() -> Fixture {
    let mut arsc = ArscFix::with_package("com.test.theme");
    let layout = arsc.add_layout("main");
    let root_id = arsc.add_id("root");
    let attr_bg = arsc.add_attr("itemBg");
    let red = arsc.add_color("red", 0xFFFF_0000);
    // Theme items: reference to a color resource, an inline color, and no item.
    let style_ref = arsc.add_style("RefTheme", 0, vec![(attr_bg, ArscVal::Ref(red))]);
    let style_inline = arsc.add_style(
        "InlineTheme",
        0,
        vec![(attr_bg, ArscVal::Int(0xFF00_FF00u32 as i32))],
    );
    let style_empty = arsc.add_style("EmptyTheme", 0, vec![]);
    Fixture {
        arsc,
        layout,
        root_id,
        attr_bg,
        style_ref,
        style_inline,
        style_empty,
    }
}

fn layout_xml(f: &Fixture) -> Vec<u8> {
    let root = AxNode::new(
        "View",
        vec![
            a(true, "id", AxVal::Ref(f.root_id)),
            a(true, "background", AxVal::Attr(f.attr_bg)),
        ],
        vec![],
    );
    build_axml(&root)
}

/// Launches the app with `theme` (or no manifest theme) and returns the
/// `ColorDrawable` color of the root view's background.
fn background_color(f: &Fixture, theme: Option<u32>) -> i32 {
    let arsc_bytes = f.arsc.build();
    let extras = vec![("res/layout/main.xml", layout_xml(f))];
    let apk = match theme {
        Some(t) => build_apk_themed(PACKAGE, ACTIVITY, arsc_bytes, extras, t),
        None => build_apk(PACKAGE, ACTIVITY, arsc_bytes, extras),
    };
    let mut e = engine_with_layout(apk, f.layout);
    e.launch_app(ACTIVITY, PACKAGE, Vec::new())
        .expect("themed activity launches");
    let act = e.fw.current_activity.expect("activity");
    let root = match e
        .invoke_instance(
            act,
            "findViewById",
            "(I)Landroid/view/View;",
            &[Value::Int(f.root_id as i32)],
        )
        .expect("findViewById")
    {
        Value::Obj(v) => v,
        other => panic!("root view missing: {other:?}"),
    };
    let bg = match e
        .invoke_instance(
            root,
            "getBackground",
            "()Landroid/graphics/drawable/Drawable;",
            &[],
        )
        .expect("getBackground")
    {
        Value::Obj(d) => d,
        other => panic!("background is not set: {other:?}"),
    };
    match e
        .invoke_instance(bg, "getColor", "()I", &[])
        .expect("ColorDrawable.getColor")
    {
        Value::Int(c) => c,
        other => panic!("color is not an int: {other:?}"),
    }
}

#[test]
fn theme_item_referencing_a_color_resource_resolves_the_background() {
    let f = fixture();
    assert_eq!(
        background_color(&f, Some(f.style_ref)),
        0xFFFF_0000u32 as i32
    );
}

#[test]
fn inline_theme_color_resolves_the_background() {
    let f = fixture();
    assert_eq!(
        background_color(&f, Some(f.style_inline)),
        0xFF00_FF00u32 as i32
    );
}

#[test]
fn attribute_absent_from_theme_keeps_the_transparent_stand_in() {
    let f = fixture();
    assert_eq!(background_color(&f, Some(f.style_empty)), 0);
}

#[test]
fn no_manifest_theme_keeps_the_transparent_stand_in() {
    let f = fixture();
    assert_eq!(background_color(&f, None), 0);
}

// Keep the sentinel constant in scope for future fixture cases.
#[allow(dead_code)]
const _NO_INDEX: u32 = NO_INDEX;
