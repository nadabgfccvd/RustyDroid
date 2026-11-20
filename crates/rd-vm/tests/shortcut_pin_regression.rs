//! Regressão do merge do Zoom: o no-op de `ShortcutManager` / `ShortcutInfo.Builder`
//! em `intrinsics.rs` não pode engolir `requestPinShortcut`. O modelo M9
//! (`m9::shortcuts`) tem de receber a chamada e gravar o atalho.
//!
//! O smoke do app C (`scripts/native-languages-mcp.py`, passo "Pin shortcut")
//! espera `shortcuts` com 1 entrada depois do pin.

#![cfg(feature = "advanced-components")]

use rd_vm::m9::Shortcut;
use rd_vm::{Engine, Value, VmConfig};

#[test]
fn request_pin_shortcut_is_recorded_by_the_m9_model() {
    let mut vm = Engine::new(vec![], VmConfig::default());
    let mgr = vm
        .heap
        .alloc_instance("Landroid/content/pm/ShortcutManager;".into(), vec![])
        .expect("manager");
    // ShortcutInfo já construído pelo modelo (o builder de M9 gera este objeto).
    let info = vm
        .heap
        .alloc_instance("Landroid/content/pm/ShortcutInfo;".into(), vec![])
        .expect("info");
    vm.fw.m9.shortcut_objects.insert(
        info,
        Shortcut {
            id: "native-proof".into(),
            short_label: "Native proof".into(),
            enabled: true,
            ..Default::default()
        },
    );

    // Mesmo caminho de despacho que o app usa (`invoke_instance` →
    // `call_instance_intrinsic_mref` → M8/M9), não a função do M9 isolada.
    let out = vm
        .invoke_instance(
            mgr,
            "requestPinShortcut",
            "(Landroid/content/pm/ShortcutInfo;Landroid/content/IntentSender;)Z",
            &[Value::Obj(info), Value::Null],
        )
        .expect("requestPinShortcut não pode falhar");

    assert_eq!(out, Value::Int(1), "pin deve ser aceito");
    let s = vm
        .fw
        .m9
        .shortcuts
        .get("native-proof")
        .expect("atalho pinado no modelo M9");
    assert!(s.pinned, "atalho deve ficar pinado");
}

/// Fluxo completo do app: `ShortcutInfo.Builder` (no-op do Zoom em
/// `intrinsics.rs`) → `build()` → `requestPinShortcut`. É o caminho que o smoke
/// do app C percorre no passo "Pin shortcut".
#[test]
fn builder_then_request_pin_reaches_the_m9_model() {
    let mut vm = Engine::new(vec![], VmConfig::default());
    let ctx = vm
        .heap
        .alloc_instance("Landroid/content/Context;".into(), vec![])
        .expect("context");
    let mgr = vm
        .heap
        .alloc_instance("Landroid/content/pm/ShortcutManager;".into(), vec![])
        .expect("manager");
    let intent = vm
        .heap
        .alloc_instance("Landroid/content/Intent;".into(), vec![])
        .expect("intent");
    let builder = vm
        .heap
        .alloc_instance("Landroid/content/pm/ShortcutInfo$Builder;".into(), vec![])
        .expect("builder");

    let id =
        rd_vm::intrinsics::alloc_string(&mut vm, "native-proof".to_string()).expect("string do id");
    vm.invoke_instance(
        builder,
        "<init>",
        "(Landroid/content/Context;Ljava/lang/String;)V",
        &[Value::Obj(ctx), Value::Obj(id)],
    )
    .expect("builder <init>");
    let label = rd_vm::intrinsics::alloc_string(&mut vm, "Native proof".to_string())
        .expect("string do label");
    vm.invoke_instance(
        builder,
        "setShortLabel",
        "(Ljava/lang/CharSequence;)Landroid/content/pm/ShortcutInfo$Builder;",
        &[Value::Obj(label)],
    )
    .expect("setShortLabel");
    vm.invoke_instance(
        builder,
        "setIntent",
        "(Landroid/content/Intent;)Landroid/content/pm/ShortcutInfo$Builder;",
        &[Value::Obj(intent)],
    )
    .expect("setIntent");
    let info = vm
        .invoke_instance(builder, "build", "()Landroid/content/pm/ShortcutInfo;", &[])
        .expect("build");
    let out = vm
        .invoke_instance(
            mgr,
            "requestPinShortcut",
            "(Landroid/content/pm/ShortcutInfo;Landroid/content/IntentSender;)Z",
            &[info, Value::Null],
        )
        .expect("requestPinShortcut");
    assert_eq!(
        out,
        Value::Int(1),
        "pin de atalho do builder deve ser aceito"
    );
}

/// Fluxo exato de `golden/native-languages/java/MainActivity.java#postShortcut`:
/// `ShortcutInfo.Builder(...).build()` e depois `setDynamicShortcuts(list)`.
/// O smoke do app C espera `shortcuts` com 1 entrada (`native-proof`).
#[test]
fn set_dynamic_shortcuts_from_builder_is_listed_by_the_m9_model() {
    use rd_vm::framework::HostObj;

    let mut vm = Engine::new(vec![], VmConfig::default());
    let ctx = vm
        .heap
        .alloc_instance("Landroid/content/Context;".into(), vec![])
        .expect("context");
    let mgr = vm
        .heap
        .alloc_instance("Landroid/content/pm/ShortcutManager;".into(), vec![])
        .expect("manager");
    let intent = vm
        .heap
        .alloc_instance("Landroid/content/Intent;".into(), vec![])
        .expect("intent");
    let builder = vm
        .heap
        .alloc_instance("Landroid/content/pm/ShortcutInfo$Builder;".into(), vec![])
        .expect("builder");
    let id = rd_vm::intrinsics::alloc_string(&mut vm, "native-proof".to_string()).expect("id");
    let label =
        rd_vm::intrinsics::alloc_string(&mut vm, "Native proof".to_string()).expect("label");

    vm.invoke_instance(
        builder,
        "<init>",
        "(Landroid/content/Context;Ljava/lang/String;)V",
        &[Value::Obj(ctx), Value::Obj(id)],
    )
    .expect("builder <init>");
    vm.invoke_instance(
        builder,
        "setShortLabel",
        "(Ljava/lang/CharSequence;)Landroid/content/pm/ShortcutInfo$Builder;",
        &[Value::Obj(label)],
    )
    .expect("setShortLabel");
    vm.invoke_instance(
        builder,
        "setIntent",
        "(Landroid/content/Intent;)Landroid/content/pm/ShortcutInfo$Builder;",
        &[Value::Obj(intent)],
    )
    .expect("setIntent");
    let info = vm
        .invoke_instance(builder, "build", "()Landroid/content/pm/ShortcutInfo;", &[])
        .expect("build");
    let Value::Obj(info_ref) = info else {
        panic!("build deve devolver ShortcutInfo")
    };

    // `Arrays.asList(info)` no app: lista host com um elemento.
    let list = vm
        .heap
        .alloc_instance("Ljava/util/ArrayList;".into(), vec![])
        .expect("list");
    vm.fw.objects.insert(
        list,
        HostObj::List {
            elems: vec![Value::Obj(info_ref)],
        },
    );

    let out = vm
        .invoke_instance(
            mgr,
            "setDynamicShortcuts",
            "(Ljava/util/List;)Z",
            &[Value::Obj(list)],
        )
        .expect("setDynamicShortcuts");
    assert_eq!(
        out,
        Value::Int(1),
        "setDynamicShortcuts deve aceitar o atalho"
    );
    let s = vm
        .fw
        .m9
        .shortcuts
        .get("native-proof")
        .expect("atalho dinâmico listado pelo M9 (smoke: shortcuts == 1)");
    assert!(s.dynamic, "atalho deve ficar como dinâmico");
}
