//! `Class` enclosing metadata comes from DEX system annotations, not class-name guesses.

use rd_dex::Dex;
use rd_testsupport::vm::*;
use rd_vm::engine::VmConfig;
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const OUTER: &str = "Ltest/Outer;";
const LOCAL_METHOD: &str = "Ltest/Outer$1Local;";
const LOCAL_CTOR: &str = "Ltest/Outer$2CtorLocal;";
const ANONYMOUS_INITIALIZER: &str = "Ltest/Outer$3;";
const CLASS: &str = "Ljava/lang/Class;";

fn fixture() -> Engine {
    let mut b = DexBuilder::new();
    let outer = b.class(OUTER, "Ljava/lang/Object;");
    let run_proto = b.proto_idx("I", vec!["Ljava/lang/String;".into()]);
    let run_idx = b.method_idx(OUTER, run_proto, "run");
    b.direct(
        outer,
        "run",
        "I",
        vec!["Ljava/lang/String;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(2, 1, 0, op11x(0x0f, 0))),
    );
    let ctor_proto = b.proto_idx("V", vec!["I".into()]);
    let ctor_idx = b.method_idx(OUTER, ctor_proto, "<init>");
    b.direct(
        outer,
        "<init>",
        "V",
        vec!["I"],
        ACC_PUBLIC,
        Some(b.code(2, 2, 0, op10x(0x0e))),
    );

    let local_method = b.class(LOCAL_METHOD, "Ljava/lang/Object;");
    b.class_annotation(
        local_method,
        2,
        "Ldalvik/annotation/EnclosingMethod;",
        vec![("value".into(), ClassAnnotationValueE::Method(run_idx))],
    );
    let local_ctor = b.class(LOCAL_CTOR, "Ljava/lang/Object;");
    b.class_annotation(
        local_ctor,
        2,
        "Ldalvik/annotation/EnclosingMethod;",
        vec![("value".into(), ClassAnnotationValueE::Method(ctor_idx))],
    );
    let anonymous_initializer = b.class(ANONYMOUS_INITIALIZER, "Ljava/lang/Object;");
    b.class_annotation(
        anonymous_initializer,
        2,
        "Ldalvik/annotation/EnclosingClass;",
        vec![(
            "value".into(),
            ClassAnnotationValueE::Type(OUTER.to_string()),
        )],
    );

    let dex = Dex::parse(b.finish()).expect("synthetic annotated DEX parses");
    Engine::new(vec![dex], VmConfig::default())
}

fn class_object(vm: &mut Engine, desc: &str) -> ObjRef {
    let name = vm.heap.alloc_string(desc.to_string()).expect("class name");
    vm.heap
        .alloc_instance(
            CLASS.to_string(),
            vec![("class_name".into(), Value::Obj(name))],
        )
        .expect("Class object")
}

fn enclosing_class_name(vm: &mut Engine, class: ObjRef) -> String {
    let enclosing = vm
        .invoke_instance(class, "getEnclosingClass", "()Ljava/lang/Class;", &[])
        .expect("getEnclosingClass");
    as_class_name(vm, enclosing)
}

fn as_class_name(vm: &Engine, value: Value) -> String {
    let Value::Obj(class) = value else {
        panic!("expected Class object, got {value:?}");
    };
    let Value::Obj(name) = vm.heap.get_field(class, "class_name").expect("class_name") else {
        panic!("class_name is not a string");
    };
    vm.heap.as_str(name).expect("descriptor").to_string()
}

#[test]
fn local_class_gets_enclosing_method_and_class_but_not_declaring_class() {
    let mut vm = fixture();
    let class = class_object(&mut vm, LOCAL_METHOD);
    let method = vm
        .invoke_instance(
            class,
            "getEnclosingMethod",
            "()Ljava/lang/reflect/Method;",
            &[],
        )
        .expect("getEnclosingMethod");
    let Value::Obj(method) = method else {
        panic!("expected enclosing Method, got {method:?}");
    };
    assert_eq!(
        vm.heap
            .as_str(match vm.heap.get_field(method, "rd_decl").unwrap() {
                Value::Obj(v) => v,
                other => panic!("rd_decl: {other:?}"),
            })
            .unwrap(),
        OUTER
    );
    assert_eq!(
        vm.heap
            .as_str(match vm.heap.get_field(method, "rd_name").unwrap() {
                Value::Obj(v) => v,
                other => panic!("rd_name: {other:?}"),
            })
            .unwrap(),
        "run"
    );
    assert_eq!(
        vm.heap
            .as_str(match vm.heap.get_field(method, "rd_sig").unwrap() {
                Value::Obj(v) => v,
                other => panic!("rd_sig: {other:?}"),
            })
            .unwrap(),
        "(Ljava/lang/String;)I"
    );

    assert_eq!(enclosing_class_name(&mut vm, class), OUTER);
    assert_eq!(
        vm.invoke_instance(class, "getDeclaringClass", "()Ljava/lang/Class;", &[],)
            .expect("getDeclaringClass"),
        Value::Null,
        "local classes have an enclosing class, but no declaring class"
    );
    assert_eq!(
        vm.invoke_instance(
            class,
            "getEnclosingConstructor",
            "()Ljava/lang/reflect/Constructor;",
            &[],
        )
        .expect("getEnclosingConstructor"),
        Value::Null
    );
}

#[test]
fn local_class_gets_enclosing_constructor_and_parameter_types() {
    let mut vm = fixture();
    let class = class_object(&mut vm, LOCAL_CTOR);
    assert_eq!(
        vm.invoke_instance(
            class,
            "getEnclosingMethod",
            "()Ljava/lang/reflect/Method;",
            &[],
        )
        .expect("getEnclosingMethod"),
        Value::Null
    );
    let constructor = vm
        .invoke_instance(
            class,
            "getEnclosingConstructor",
            "()Ljava/lang/reflect/Constructor;",
            &[],
        )
        .expect("getEnclosingConstructor");
    let Value::Obj(constructor) = constructor else {
        panic!("expected enclosing Constructor, got {constructor:?}");
    };
    assert_eq!(
        vm.invoke_instance(constructor, "getParameterCount", "()I", &[])
            .expect("getParameterCount"),
        Value::Int(1)
    );
    let params = vm
        .invoke_instance(
            constructor,
            "getParameterTypes",
            "()[Ljava/lang/Class;",
            &[],
        )
        .expect("getParameterTypes");
    let Value::Obj(params) = params else {
        panic!("expected parameter array, got {params:?}");
    };
    let param = match vm.heap.as_array_elems(params).unwrap().0.first() {
        Some(Value::Obj(param)) => *param,
        other => panic!("expected primitive Class element, got {other:?}"),
    };
    assert_eq!(as_class_name(&vm, Value::Obj(param)), "I");
    assert_eq!(enclosing_class_name(&mut vm, class), OUTER);
}

#[test]
fn enclosing_class_annotation_and_top_level_nulls_are_respected() {
    let mut vm = fixture();
    let anonymous = class_object(&mut vm, ANONYMOUS_INITIALIZER);
    assert_eq!(enclosing_class_name(&mut vm, anonymous), OUTER);
    assert_eq!(
        vm.invoke_instance(
            anonymous,
            "getEnclosingMethod",
            "()Ljava/lang/reflect/Method;",
            &[],
        )
        .expect("getEnclosingMethod"),
        Value::Null
    );
    assert_eq!(
        vm.invoke_instance(
            anonymous,
            "getEnclosingConstructor",
            "()Ljava/lang/reflect/Constructor;",
            &[],
        )
        .expect("getEnclosingConstructor"),
        Value::Null
    );
    assert_eq!(
        vm.invoke_instance(anonymous, "getDeclaringClass", "()Ljava/lang/Class;", &[],)
            .expect("getDeclaringClass"),
        Value::Null
    );
    let outer = class_object(&mut vm, OUTER);
    assert_eq!(
        vm.invoke_instance(
            outer,
            "getEnclosingMethod",
            "()Ljava/lang/reflect/Method;",
            &[],
        )
        .expect("top-level getEnclosingMethod"),
        Value::Null
    );
    assert_eq!(
        vm.invoke_instance(
            outer,
            "getEnclosingConstructor",
            "()Ljava/lang/reflect/Constructor;",
            &[],
        )
        .expect("top-level getEnclosingConstructor"),
        Value::Null
    );
    assert_eq!(
        vm.invoke_instance(outer, "getEnclosingClass", "()Ljava/lang/Class;", &[],)
            .expect("top-level getEnclosingClass"),
        Value::Null
    );
}
