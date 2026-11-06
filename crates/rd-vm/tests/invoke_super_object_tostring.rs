//! Regressão Scanner do Comprix (MEDIA-03): o `AbstractFuture.toString()` do
//! Guava chama `super.toString()` sobre `this`, cuja superclasse direta é
//! `Object`. Esse `invoke-super Object.toString()` reentrava no próprio override
//! e falhava com NOT_FOUND `Ljava/lang/Object;->toString()Ljava/lang/String;`.
//! A busca precisa começar na superclasse do método em execução (identidade).

use rd_testsupport::vm::*;
use rd_vm::Value;

#[test]
fn invoke_super_object_to_string_uses_identity_default() {
    let mut b = DexBuilder::new();

    // class SupTs { public String toString() { return super.toString(); } }
    let cls = b.class("LSupTs;", "Ljava/lang/Object;");
    let proto = b.proto_idx("Ljava/lang/String;", vec![]);
    let object_to_string = b.method_idx("Ljava/lang/Object;", proto, "toString");
    let mut code = op35c(0x6f, 1, object_to_string, [1, 0, 0, 0, 0]); // invoke-super {this}
    code.extend(op11x(0x0c, 0)); // move-result-object v0
    code.extend(op11x(0x11, 0)); // return-object v0
    let blob = b.code(2, 1, 1, code);
    b.r#virtual(
        cls,
        "toString",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC,
        Some(blob),
    );

    let mut e = engine_of(&b);
    e.ensure_initialized("LSupTs;").expect("init SupTs");
    let obj = e.new_dex_instance_pub("LSupTs;").expect("alloc SupTs");
    let got = e
        .invoke_instance(obj, "toString", "()Ljava/lang/String;", &[])
        .expect("super.toString() deve resolver o default de Object");
    let Value::Obj(s) = got else {
        panic!("toString deve devolver String, veio {got:?}");
    };
    let text = e.heap.as_str(s).expect("string").to_string();
    assert!(
        text.starts_with("SupTs@"),
        "default de Object.toString é Classe@hash, veio {text:?}"
    );
}
