//! `java.lang.reflect.Proxy` regression.
//!
//! Real apps (F-Droid and NewPipe) reach ACRA before `onCreate` and stop on
//! `Proxy.newProxyInstance`. The proxy object must implement the requested
//! interface and forward calls—including `Object` methods—to the supplied
//! `InvocationHandler`, boxing primitive arguments/returns on the reflective
//! boundary.

use rd_dex::Dex;
use rd_testsupport::vm::*;
use rd_vm::{Engine, ObjRef, Value, VmConfig};

const IFACE: &str = "LProxyTarget;";
const HANDLER: &str = "LProxyHandler;";
const ACC_PUBLIC: u32 = 0x1;
const ACC_ABSTRACT: u32 = 0x400;
const ACC_INTERFACE: u32 = 0x200;

fn engine() -> Engine {
    let mut b = DexBuilder::new();

    // The interface only needs a method reference; the proxy implements it.
    let iface = b.class(IFACE, "Ljava/lang/Object;");
    b.set_access(iface, ACC_INTERFACE);
    b.r#virtual(iface, "ping", "I", vec![], ACC_PUBLIC | ACC_ABSTRACT, None);

    // Handler.invoke returns the preallocated Integer stored in `result`.
    let result_field = b.field_idx(HANDLER, "Ljava/lang/Integer;", "result");
    let mut units = op22c(0x54, 0, 0, result_field); // iget-object v0, p0, result
    units.extend(op11x(0x11, 0)); // return-object v0
    let handler_code = b.code(4, 4, 0, units);
    let handler = b.class(HANDLER, "Ljava/lang/Object;");
    b.instance_field(handler, "result", "Ljava/lang/Integer;");
    b.r#virtual(
        handler,
        "invoke",
        "Ljava/lang/Object;",
        vec![
            "Ljava/lang/Object;",
            "Ljava/lang/reflect/Method;",
            "[Ljava/lang/Object;",
        ],
        ACC_PUBLIC,
        Some(handler_code),
    );

    let dex = Dex::parse(b.finish()).expect("DEX");
    Engine::new(vec![dex], VmConfig::default())
}

fn class_obj(vm: &mut Engine, desc: &str) -> ObjRef {
    let s = vm.heap.alloc_string(desc.to_string()).expect("alloc name");
    vm.heap
        .alloc_instance(
            "Ljava/lang/Class;".to_string(),
            vec![("class_name".to_string(), Value::Obj(s))],
        )
        .expect("alloc Class")
}

fn proxy(vm: &mut Engine) -> ObjRef {
    let iface_class = class_obj(vm, IFACE);
    let classes = vm
        .heap
        .alloc_array(rd_vm::heap::ElemKind::Obj, 1, "Ljava/lang/Class;")
        .expect("class array");
    *vm.heap.array_elems_mut(classes).expect("class array elems") = vec![Value::Obj(iface_class)];
    let result = vm
        .heap
        .alloc_instance(
            "Ljava/lang/Integer;".to_string(),
            vec![("value".to_string(), Value::Int(42))],
        )
        .expect("Integer result");
    let handler = vm
        .heap
        .alloc_instance(
            HANDLER.to_string(),
            vec![("result".to_string(), Value::Obj(result))],
        )
        .expect("handler");

    match vm
        .invoke_static(
            "Ljava/lang/reflect/Proxy;",
            "newProxyInstance",
            "(Ljava/lang/ClassLoader;[Ljava/lang/Class;Ljava/lang/reflect/InvocationHandler;)Ljava/lang/Object;",
            &[Value::Null, Value::Obj(classes), Value::Obj(handler)],
        )
        .expect("Proxy.newProxyInstance")
    {
        Value::Obj(p) => p,
        other => panic!("proxy is not an object: {other:?}"),
    }
}

#[test]
fn proxy_dispatches_interface_and_object_methods_to_handler() {
    let mut vm = engine();
    let p = proxy(&mut vm);

    assert_eq!(
        vm.invoke_instance(p, "ping", "()I", &[]).expect("ping"),
        Value::Int(42)
    );
    assert_eq!(
        vm.invoke_instance(p, "hashCode", "()I", &[])
            .expect("hashCode"),
        Value::Int(42)
    );
    assert_eq!(
        vm.invoke_instance(p, "equals", "(Ljava/lang/Object;)Z", &[Value::Obj(p)])
            .expect("equals"),
        Value::Int(1)
    );

    let iface_class = class_obj(&mut vm, IFACE);
    assert_eq!(
        vm.invoke_instance(
            iface_class,
            "isInstance",
            "(Ljava/lang/Object;)Z",
            &[Value::Obj(p)]
        )
        .expect("Class.isInstance"),
        Value::Int(1)
    );
    assert_eq!(
        vm.invoke_instance(
            iface_class,
            "cast",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            &[Value::Obj(p)]
        )
        .expect("Class.cast"),
        Value::Obj(p)
    );
}
