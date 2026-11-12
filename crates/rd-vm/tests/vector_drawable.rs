//! `<vector>` dentro de um APK real (issue: apps AppCompat não abrem sem ele).
//!
//! O `AppCompatDrawableManager.checkVectorDrawableSetup` faz
//! `getDrawable(R.drawable.abc_vector_test)` e lança `IllegalStateException` se
//! o resultado não for uma instância de `android.graphics.drawable.VectorDrawable`.
//! Este teste garante que o caminho completo — ARSC → arquivo AXML → spec →
//! objeto de heap — devolve a classe e o intrínseco certos.

mod common;

use common::apkfix::{self, AxNode, AxVal};
use rd_vm::{Engine, Value, VmConfig};

const VECTOR_XML: &str = "res/drawable/ic_test.xml";

fn vector_axml() -> Vec<u8> {
    let root = AxNode::new(
        "vector",
        vec![
            apkfix::a(true, "width", AxVal::Dim { raw: (48 << 8) | 1 }),
            apkfix::a(true, "height", AxVal::Dim { raw: (48 << 8) | 1 }),
            apkfix::a(true, "viewportWidth", AxVal::Int(24)),
            apkfix::a(true, "viewportHeight", AxVal::Int(24)),
        ],
        vec![
            AxNode::new(
                "path",
                vec![
                    apkfix::a(
                        true,
                        "pathData",
                        AxVal::Str("M0,0 L24,0 L24,24 Z".to_string()),
                    ),
                    apkfix::a(true, "fillColor", AxVal::Int(0xFF00_FF00u32 as i32)),
                ],
                vec![],
            ),
            AxNode::new(
                "group",
                vec![
                    apkfix::a(true, "translateX", AxVal::Int(2)),
                    apkfix::a(true, "scaleX", AxVal::Int(2)),
                    apkfix::a(true, "scaleY", AxVal::Int(2)),
                ],
                vec![AxNode::new(
                    "path",
                    vec![
                        apkfix::a(true, "pathData", AxVal::Str("M1,1 L2,1 L2,2 Z".to_string())),
                        apkfix::a(true, "fillColor", AxVal::Int(0xFFFF_0000u32 as i32)),
                    ],
                    vec![],
                )],
            ),
        ],
    );
    apkfix::build_axml(&root)
}

fn apk_with_vector() -> Vec<u8> {
    let mut arsc = apkfix::ArscFix::new();
    arsc.add_drawable_file("ic_test");
    let mut zip = apkfix::ZipFix::new();
    zip.add(
        "AndroidManifest.xml",
        apkfix::build_manifest_axml("com.test.vec", "Main"),
    );
    zip.add("resources.arsc", arsc.build());
    zip.add(VECTOR_XML, vector_axml());
    zip.build()
}

fn engine_with_apk(bytes: Vec<u8>) -> Engine {
    let mut vm = Engine::new(vec![], VmConfig::default());
    let apk = rd_apk::Apk::from_bytes(bytes).expect("APK sintético parseia");
    vm.set_resources(apk);
    vm
}

fn get_drawable(vm: &mut Engine, id: u32) -> Value {
    let ctx = vm
        .heap
        .alloc_instance("Landroid/content/Context;".into(), vec![])
        .expect("context");
    vm.invoke_instance(
        ctx,
        "getDrawable",
        "(I)Landroid/graphics/drawable/Drawable;",
        &[Value::Int(id as i32)],
    )
    .expect("getDrawable de um <vector> não pode falhar")
}

#[test]
fn get_drawable_returns_a_vector_drawable_instance() {
    let bytes = apk_with_vector();
    let mut vm = engine_with_apk(bytes);
    let id = 0x7f00_0000 | (1 << 16); // primeiro tipo inserido = drawable, entrada 0
    let d = get_drawable(&mut vm, id);
    let Value::Obj(obj) = d else {
        panic!("esperava um objeto Drawable, veio {d:?}");
    };
    assert_eq!(
        vm.heap.class_of(obj).unwrap(),
        "Landroid/graphics/drawable/VectorDrawable;",
        "AppCompat exige exatamente esta classe no self-check"
    );
}

#[test]
fn vector_drawable_reports_its_intrinsic_size() {
    let bytes = apk_with_vector();
    let mut vm = engine_with_apk(bytes);
    let id = 0x7f00_0000 | (1 << 16);
    let Value::Obj(d) = get_drawable(&mut vm, id) else {
        panic!("esperava Obj");
    };
    assert_eq!(
        vm.invoke_instance(d, "getIntrinsicWidth", "()I", &[])
            .unwrap(),
        Value::Int(48),
        "android:width=\"48dp\""
    );
    assert_eq!(
        vm.invoke_instance(d, "getIntrinsicHeight", "()I", &[])
            .unwrap(),
        Value::Int(48)
    );
}

#[test]
fn vector_drawable_is_a_drawable_for_instanceof() {
    // O self-check do AppCompat é `d instanceof VectorDrawable`; o do dump é
    // `instanceof Drawable`. A hierarquia do classpath tem de ter as duas.
    let bytes = apk_with_vector();
    let mut vm = engine_with_apk(bytes);
    let id = 0x7f00_0000 | (1 << 16);
    let Value::Obj(d) = get_drawable(&mut vm, id) else {
        panic!("esperava Obj");
    };
    let class = vm.heap.class_of(d).unwrap().to_string();
    assert!(
        vm.cp
            .is_subtype(&class, "Landroid/graphics/drawable/Drawable;"),
        "VectorDrawable precisa ser subtype de Drawable"
    );
}
