//! `Resources.getColorStateList` com seletor XML (`res/color/*.xml`) num APK real.
//!
//! Antes: lançava `NOT_IMPLEMENTED` para qualquer seletor. Agora o seletor é
//! lido do próprio APK, com os IDs de `state_*` vindos do resource map do
//! fixture (`0x0101_xxxx`), não de tabela no código.

mod common;

use common::apkfix::{self, AxNode, AxVal};
use rd_vm::{Engine, Value, VmConfig};

const SEL: &str = "res/color/btn.xml";
const PRESSED_ARGB: i32 = 0xFFFF_0000u32 as i32; // vermelho
const DEFAULT_ARGB: i32 = 0xFF00_FF00u32 as i32; // verde

fn selector_axml() -> Vec<u8> {
    let root = AxNode::new(
        "selector",
        vec![],
        vec![
            AxNode::new(
                "item",
                vec![
                    apkfix::a(true, "state_pressed", AxVal::Bool(true)),
                    apkfix::a(true, "color", AxVal::Int(PRESSED_ARGB)),
                ],
                vec![],
            ),
            AxNode::new(
                "item",
                vec![apkfix::a(true, "color", AxVal::Int(DEFAULT_ARGB))],
                vec![],
            ),
        ],
    );
    apkfix::build_axml(&root)
}

fn apk_with_selector() -> Vec<u8> {
    let mut arsc = apkfix::ArscFix::new();
    arsc.add_file("color", "btn");
    let mut zip = apkfix::ZipFix::new();
    zip.add(
        "AndroidManifest.xml",
        apkfix::build_manifest_axml("com.test.sel", "Main"),
    );
    zip.add("resources.arsc", arsc.build());
    zip.add(SEL, selector_axml());
    zip.build()
}

#[test]
fn selector_xml_becomes_a_color_state_list_with_real_state_ids() {
    let mut arsc = apkfix::ArscFix::new();
    let id = arsc.add_file("color", "btn");
    let mut vm = Engine::new(vec![], VmConfig::default());
    let apk = rd_apk::Apk::from_bytes(apk_with_selector()).expect("APK sintético parseia");
    vm.set_resources(apk);
    let ctx = vm
        .heap
        .alloc_instance("Landroid/content/Context;".into(), vec![])
        .expect("context");
    let res = vm
        .invoke_instance(
            ctx,
            "getResources",
            "()Landroid/content/res/Resources;",
            &[],
        )
        .expect("getResources");
    let Value::Obj(res) = res else {
        panic!("getResources deve devolver objeto")
    };
    let csl = vm
        .invoke_instance(
            res,
            "getColorStateList",
            "(I)Landroid/content/res/ColorStateList;",
            &[Value::Int(id as i32)],
        )
        .expect("getColorStateList de seletor XML não pode falhar");
    let Value::Obj(csl) = csl else {
        panic!("ColorStateList deve ser objeto")
    };
    let default = vm
        .invoke_instance(csl, "getDefaultColor", "()I", &[])
        .expect("getDefaultColor");
    // Sem estado: o item sem `state_*` é o curinga, que vence o padrão.
    assert_eq!(default, Value::Int(DEFAULT_ARGB));

    // O estado pressionado precisa de um ID real do resource map do APK
    // (`0x0101_xxxx`). Sem `state_pressed` no conjunto, o curinga responde.
    // ID real de `state_pressed` no resource map do próprio AXML do fixture.
    let pressed_id = pressed_state_id_in_fixture();
    let color_for = |vm: &mut Engine, states: Vec<i32>| {
        let arr = vm
            .heap
            .alloc_array(rd_vm::heap::ElemKind::Int, states.len(), "[I")
            .expect("int[] de estados");
        vm.heap
            .array_elems_mut(arr)
            .expect("elementos do int[]")
            .extend(states.iter().copied().map(Value::Int));
        vm.invoke_instance(
            csl,
            "getColorForState",
            "([II)I",
            &[Value::Obj(arr), Value::Int(0)],
        )
        .expect("getColorForState")
    };
    assert_eq!(
        color_for(&mut vm, vec![pressed_id]),
        Value::Int(PRESSED_ARGB)
    );
    assert_eq!(color_for(&mut vm, vec![]), Value::Int(DEFAULT_ARGB));
}

/// Lê do AXML do fixture o `res_id` de `state_pressed` (não um número fixo).
fn pressed_state_id_in_fixture() -> i32 {
    let doc = rd_apk::axml::parse(&selector_axml()).expect("axml do seletor");
    let item = doc
        .root
        .children
        .iter()
        .find(|c| c.android_attr("state_pressed").is_some())
        .expect("item com state_pressed");
    let id = item
        .android_attr("state_pressed")
        .and_then(|a| a.res_id)
        .expect("state_pressed com resource id no resource map");
    assert!(id >= 0x0101_0000, "id de atributo android: 0x{id:x}");
    id as i32
}
