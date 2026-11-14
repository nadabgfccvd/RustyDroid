//! `<shape>` dentro de um APK real → `GradientDrawable`.
//!
//! Fundos de botões e cards são quase sempre `res/drawable/*.xml` com raiz
//! `<shape>`. O caminho ARSC → AXML → objeto de heap tem de devolver a classe
//! certa, com geometria, cor e tamanho intrínseco legíveis pela API pública.

mod common;

use common::apkfix::{self, AxNode, AxVal};
use rd_vm::{Engine, Value, VmConfig};

const SHAPE_XML: &str = "res/drawable/bg_button.xml";
/// Primeiro tipo inserido = drawable, entrada 0.
const SHAPE_ID: u32 = 0x7f00_0000 | (1 << 16);

/// Dimensão AXML em px (unidade 0), independente da densidade do device.
fn px(v: u32) -> AxVal {
    AxVal::Dim { raw: v << 8 }
}

fn shape_axml() -> Vec<u8> {
    let root = AxNode::new(
        "shape",
        vec![apkfix::a(true, "shape", AxVal::Int(0))],
        vec![
            AxNode::new(
                "solid",
                vec![apkfix::a(true, "color", AxVal::Int(0xFF33_66CCu32 as i32))],
                vec![],
            ),
            AxNode::new(
                "stroke",
                vec![
                    apkfix::a(true, "width", px(2)),
                    apkfix::a(true, "color", AxVal::Int(0xFF00_0000u32 as i32)),
                ],
                vec![],
            ),
            AxNode::new("corners", vec![apkfix::a(true, "radius", px(12))], vec![]),
            AxNode::new(
                "size",
                vec![
                    apkfix::a(true, "width", px(120)),
                    apkfix::a(true, "height", px(40)),
                ],
                vec![],
            ),
        ],
    );
    apkfix::build_axml(&root)
}

fn engine_with_shape() -> Engine {
    let mut arsc = apkfix::ArscFix::new();
    arsc.add_drawable_file("bg_button");
    let mut zip = apkfix::ZipFix::new();
    zip.add(
        "AndroidManifest.xml",
        apkfix::build_manifest_axml("com.test.shape", "Main"),
    );
    zip.add("resources.arsc", arsc.build());
    zip.add(SHAPE_XML, shape_axml());
    let mut vm = Engine::new(vec![], VmConfig::default());
    let apk = rd_apk::Apk::from_bytes(zip.build()).expect("APK sintético parseia");
    vm.set_resources(apk);
    vm
}

fn shape_drawable(vm: &mut Engine) -> rd_vm::ObjRef {
    let ctx = vm
        .heap
        .alloc_instance("Landroid/content/Context;".into(), vec![])
        .expect("context");
    let d = vm
        .invoke_instance(
            ctx,
            "getDrawable",
            "(I)Landroid/graphics/drawable/Drawable;",
            &[Value::Int(SHAPE_ID as i32)],
        )
        .expect("getDrawable de um <shape> não pode falhar");
    let Value::Obj(obj) = d else {
        panic!("esperava um objeto Drawable, veio {d:?}");
    };
    obj
}

#[test]
fn get_drawable_of_shape_returns_gradient_drawable() {
    let mut vm = engine_with_shape();
    let d = shape_drawable(&mut vm);
    let class = vm.heap.class_of(d).unwrap().to_string();
    assert_eq!(class, "Landroid/graphics/drawable/GradientDrawable;");
    assert!(
        vm.cp
            .is_subtype(&class, "Landroid/graphics/drawable/Drawable;"),
        "GradientDrawable precisa ser subtype de Drawable"
    );
}

#[test]
fn shape_attributes_are_readable_through_the_public_api() {
    let mut vm = engine_with_shape();
    let d = shape_drawable(&mut vm);
    let call = |vm: &mut Engine, name: &str, sig: &str| vm.invoke_instance(d, name, sig, &[]);
    assert_eq!(call(&mut vm, "getShape", "()I").unwrap(), Value::Int(0));
    assert_eq!(
        call(&mut vm, "getIntrinsicWidth", "()I").unwrap(),
        Value::Int(120)
    );
    assert_eq!(
        call(&mut vm, "getIntrinsicHeight", "()I").unwrap(),
        Value::Int(40)
    );
    assert_eq!(
        call(&mut vm, "getCornerRadius", "()F").unwrap(),
        Value::Float(12.0)
    );
    // Cantos arredondados + traço: não é opaco (PixelFormat.TRANSLUCENT).
    assert_eq!(call(&mut vm, "getOpacity", "()I").unwrap(), Value::Int(-3));
}

#[test]
fn programmatic_gradient_drawable_tracks_shape_and_size() {
    let mut vm = Engine::new(vec![], VmConfig::default());
    let d = vm
        .heap
        .alloc_instance(
            "Landroid/graphics/drawable/GradientDrawable;".into(),
            vec![],
        )
        .expect("alloc");
    vm.invoke_instance(d, "<init>", "()V", &[]).expect("<init>");
    vm.invoke_instance(d, "setShape", "(I)V", &[Value::Int(1)])
        .expect("setShape");
    vm.invoke_instance(d, "setSize", "(II)V", &[Value::Int(30), Value::Int(20)])
        .expect("setSize");
    vm.invoke_instance(d, "setCornerRadius", "(F)V", &[Value::Float(4.0)])
        .expect("setCornerRadius");
    assert_eq!(
        vm.invoke_instance(d, "getShape", "()I", &[]).unwrap(),
        Value::Int(1)
    );
    assert_eq!(
        vm.invoke_instance(d, "getIntrinsicWidth", "()I", &[])
            .unwrap(),
        Value::Int(30)
    );
    assert_eq!(
        vm.invoke_instance(d, "getCornerRadius", "()F", &[])
            .unwrap(),
        Value::Float(4.0)
    );
}
