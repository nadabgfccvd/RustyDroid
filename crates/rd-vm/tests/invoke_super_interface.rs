//! Regressão F-Droid (ktor `HttpClientEngineBase.install`): `invoke-super` cujo
//! método referenciado é um DEFAULT METHOD de interface precisa começar a busca
//! na própria interface. Antes subia para `Object` e falhava com NOT_FOUND.

use rd_testsupport::vm::*;
use rd_vm::Value;

const ACC_INTERFACE: u32 = 0x0200;
const ACC_ABSTRACT: u32 = 0x0400;

#[test]
fn invoke_super_to_interface_default_method_resolves_in_the_interface() {
    let mut b = DexBuilder::new();

    // interface Iface { default int value() { return 7; } }
    let iface = b.class("LIface;", "Ljava/lang/Object;");
    b.set_access(iface, ACC_INTERFACE | ACC_ABSTRACT);
    let mut value_code = op11n(0x12, 0, 7); // const/4 v0, #7
    value_code.extend(op11x(0x0f, 0)); // return v0
    let value_blob = b.code(2, 1, 0, value_code);
    b.r#virtual(iface, "value", "I", vec![], ACC_PUBLIC, Some(value_blob));

    // class Impl { int bridge() { return Iface.super.value(); } }  (invoke-super)
    let imp = b.class("LImpl;", "Ljava/lang/Object;");
    let proto = b.proto_idx("I", vec![]);
    let value_m = b.method_idx("LIface;", proto, "value");
    let mut code = op35c(0x6f, 1, value_m, [1, 0, 0, 0, 0]); // invoke-super {this}
    code.extend(op11x(0x0a, 0)); // move-result v0
    code.extend(op11x(0x0f, 0)); // return v0
    let bridge_blob = b.code(2, 1, 1, code);
    b.r#virtual(imp, "bridge", "I", vec![], ACC_PUBLIC, Some(bridge_blob));

    let mut e = engine_of(&b);
    e.ensure_initialized("LImpl;").expect("init Impl");
    let obj = e.new_dex_instance_pub("LImpl;").expect("alloc Impl");
    let got = e
        .invoke_instance(obj, "bridge", "()I", &[])
        .expect("invoke-super deve resolver o default da interface");
    assert_eq!(got, Value::Int(7));
}
