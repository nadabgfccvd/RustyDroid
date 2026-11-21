//! Application/ContextWrapper lifecycle regressions found with NewPipe 0.29.3.
//! The real application overrides attachBaseContext and calls super using an
//! Application method reference (invoke-super/range).

use rd_testsupport::vm::*;
use rd_vm::{err::VmExit, Engine, Value};

const APP: &str = "Landroid/app/Application;";
const CONTEXT: &str = "Landroid/content/Context;";
const WRAPPER: &str = "Landroid/content/ContextWrapper;";
const ATTACH_SIG: &str = "(Landroid/content/Context;)V";
const GET_SIG: &str = "()Landroid/content/Context;";

fn app_fixture() -> Engine {
    let mut b = DexBuilder::new();
    let app = b.class("LProbeApp;", APP);
    b.static_field(app, "phase", "I", SVal::Int(0));
    b.static_field(app, "before", CONTEXT, SVal::Int(0));
    b.static_field(app, "after", CONTEXT, SVal::Int(0));
    let phase = b.field_idx("LProbeApp;", "I", "phase");
    let before = b.field_idx("LProbeApp;", CONTEXT, "before");
    let after = b.field_idx("LProbeApp;", CONTEXT, "after");
    let attach_proto = b.proto_idx("V", vec![CONTEXT.into()]);
    let attach = b.method_idx(APP, attach_proto, "attachBaseContext");
    let get_proto = b.proto_idx(CONTEXT, vec![]);
    let get = b.method_idx(APP, get_proto, "getBaseContext");

    // p0=v1, p1=v2. Capture the base before and after super.attachBaseContext.
    let mut code = op35c(0x6e, 1, get, [1, 0, 0, 0, 0]);
    code.extend(op11x(0x0c, 0));
    code.extend(op21c(0x69, 0, before));
    code.extend([0x0275, attach, 1]); // invoke-super/range {v1..v2}, Application.attachBaseContext
    code.extend(op35c(0x6e, 1, get, [1, 0, 0, 0, 0]));
    code.extend(op11x(0x0c, 0));
    code.extend(op21c(0x69, 0, after));
    code.extend(op11n(0x12, 0, 1));
    code.extend(op21c(0x67, 0, phase));
    code.extend(op10x(0x0e));
    b.r#virtual(
        app,
        "attachBaseContext",
        "V",
        vec![CONTEXT],
        ACC_PUBLIC,
        Some(b.code(3, 2, 2, code)),
    );
    let mut on_create = op21c(0x60, 0, phase);
    on_create.extend(op22b(0xd8, 0, 0, 1));
    on_create.extend(op21c(0x67, 0, phase));
    on_create.extend(op10x(0x0e));
    b.r#virtual(
        app,
        "onCreate",
        "V",
        vec![],
        ACC_PUBLIC,
        Some(b.code(2, 1, 0, on_create)),
    );
    b.class("LProbeActivity;", "Landroid/app/Activity;");

    // A virtual call named through the platform Application type must still
    // execute the guest override, not the platform implementation directly.
    let driver = b.class("LAttachDriver;", "Ljava/lang/Object;");
    let mut code = op35c(0x6e, 2, attach, [0, 1, 0, 0, 0]);
    code.extend(op10x(0x0e));
    b.direct(
        driver,
        "call",
        "V",
        vec![APP, CONTEXT],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 2, 2, code)),
    );
    Engine::new(
        vec![rd_dex::Dex::parse(b.finish()).unwrap()],
        Default::default(),
    )
}

fn static_value(vm: &Engine, name: &str) -> Value {
    vm.statics
        .get(&("LProbeApp;".to_string(), name.to_string()))
        .cloned()
        .expect("fixture static")
}

#[test]
fn launch_runs_override_and_super_before_oncreate_with_real_base() {
    let mut vm = app_fixture();
    vm.set_application_class("LProbeApp;".to_string());
    vm.launch_app("LProbeActivity;", "test.app", Vec::new())
        .expect("attach and launch");
    assert_eq!(static_value(&vm, "phase"), Value::Int(2));
    assert_eq!(static_value(&vm, "before").as_ref().unwrap(), None);
    let base = static_value(&vm, "after").as_ref().unwrap().unwrap();
    assert_eq!(vm.heap.class_of(base).unwrap(), "Landroid/app/ContextImpl;");
}

#[test]
fn virtual_platform_reference_dispatches_to_application_override() {
    let mut vm = app_fixture();
    let app = vm.heap.alloc_instance("LProbeApp;".into(), vec![]).unwrap();
    let base = vm.heap.alloc_instance(CONTEXT.into(), vec![]).unwrap();
    vm.invoke_static(
        "LAttachDriver;",
        "call",
        "(Landroid/app/Application;Landroid/content/Context;)V",
        &[Value::Obj(app), Value::Obj(base)],
    )
    .expect("virtual attach");
    assert_eq!(static_value(&vm, "phase"), Value::Int(1));
    assert_eq!(static_value(&vm, "after"), Value::Obj(base));
}

#[test]
fn context_wrapper_preserves_null_base_and_rejects_reattachment() {
    let mut vm = Engine::new(Vec::new(), Default::default());
    let wrapper = vm.heap.alloc_instance(WRAPPER.into(), vec![]).unwrap();
    let base = vm.heap.alloc_instance(CONTEXT.into(), vec![]).unwrap();
    let second = vm.heap.alloc_instance(CONTEXT.into(), vec![]).unwrap();
    vm.invoke_instance(wrapper, "<init>", ATTACH_SIG, &[Value::Null])
        .expect("constructor");
    assert_eq!(
        vm.invoke_instance(wrapper, "getBaseContext", GET_SIG, &[])
            .unwrap(),
        Value::Null
    );
    // Dalvik's integer zero is also a null reference. Attaching null leaves
    // the wrapper unattached and does not forbid the later non-null attach.
    vm.invoke_instance(wrapper, "attachBaseContext", ATTACH_SIG, &[Value::Int(0)])
        .expect("attach null");
    assert_eq!(
        vm.invoke_instance(wrapper, "getBaseContext", GET_SIG, &[])
            .unwrap(),
        Value::Null
    );
    vm.invoke_instance(
        wrapper,
        "attachBaseContext",
        ATTACH_SIG,
        &[Value::Obj(base)],
    )
    .expect("first attach");
    assert_eq!(
        vm.invoke_instance(wrapper, "getBaseContext", GET_SIG, &[])
            .unwrap(),
        Value::Obj(base)
    );
    let err = vm
        .invoke_instance(
            wrapper,
            "attachBaseContext",
            ATTACH_SIG,
            &[Value::Obj(second)],
        )
        .expect_err("second attach must fail");
    assert!(matches!(err, VmExit::Exception(e) if e.class == "Ljava/lang/IllegalStateException;"));
    assert_eq!(
        vm.invoke_instance(wrapper, "getBaseContext", GET_SIG, &[])
            .unwrap(),
        Value::Obj(base),
        "failed attach must not change the original base"
    );
}

#[test]
fn default_application_is_attached_and_an_override_without_super_is_not() {
    for omit_super in [false, true] {
        let mut b = DexBuilder::new();
        b.class("LProbeActivity;", "Landroid/app/Activity;");
        if omit_super {
            let app = b.class("LNoSuperApp;", APP);
            b.r#virtual(
                app,
                "attachBaseContext",
                "V",
                vec![CONTEXT],
                ACC_PUBLIC,
                Some(b.code(2, 2, 0, op10x(0x0e))),
            );
        }
        let mut vm = Engine::new(
            vec![rd_dex::Dex::parse(b.finish()).unwrap()],
            Default::default(),
        );
        if omit_super {
            vm.set_application_class("LNoSuperApp;".into());
        }
        let activity = vm
            .launch_app("LProbeActivity;", "test.app", Vec::new())
            .expect("launch");
        let app = match vm
            .invoke_instance(
                activity,
                "getApplication",
                "()Landroid/app/Application;",
                &[],
            )
            .expect("getApplication")
        {
            Value::Obj(app) => app,
            other => panic!("expected Application object, got {other:?}"),
        };
        let base = vm
            .invoke_instance(app, "getBaseContext", GET_SIG, &[])
            .expect("getBaseContext");
        if omit_super {
            assert_eq!(base, Value::Null);
        } else {
            let Value::Obj(base) = base else {
                panic!("default Application should be attached, got {base:?}");
            };
            assert_eq!(vm.heap.class_of(base).unwrap(), "Landroid/app/ContextImpl;");
        }
    }
}

#[test]
fn virtual_platform_get_base_context_respects_dex_override() {
    let mut b = DexBuilder::new();
    let app = b.class("LGetterApp;", APP);
    b.r#virtual(
        app,
        "getBaseContext",
        CONTEXT,
        vec![],
        ACC_PUBLIC,
        Some(b.code(1, 1, 0, op11x(0x11, 0))), // return this
    );
    let get_proto = b.proto_idx(CONTEXT, vec![]);
    let get = b.method_idx(APP, get_proto, "getBaseContext");
    let driver = b.class("LGetDriver;", "Ljava/lang/Object;");
    let mut code = op35c(0x6e, 1, get, [1, 0, 0, 0, 0]);
    code.extend(op11x(0x0c, 0));
    code.extend(op11x(0x11, 0));
    b.direct(
        driver,
        "call",
        CONTEXT,
        vec![APP],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 1, code)),
    );
    let mut vm = Engine::new(
        vec![rd_dex::Dex::parse(b.finish()).unwrap()],
        Default::default(),
    );
    let app = vm
        .heap
        .alloc_instance("LGetterApp;".into(), vec![])
        .unwrap();
    assert_eq!(
        vm.invoke_static(
            "LGetDriver;",
            "call",
            "(Landroid/app/Application;)Landroid/content/Context;",
            &[Value::Obj(app)],
        )
        .unwrap(),
        Value::Obj(app)
    );
}
