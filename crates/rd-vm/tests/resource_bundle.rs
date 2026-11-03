//! `java.util.ResourceBundle` — o bundle *class-based* que o `PrettyTime` do
//! NewPipe carrega em `Application.onCreate` (`Resources_pt`, `Resources_en`,
//! … são classes DEX que estendem `java.util.ListResourceBundle`).
//!
//! O teste monta duas variantes de bundle (`LBundle;` e `LBundle_pt;`) e cobre
//! a ordem de candidatos do JDK, a leitura por `getString`/`getObject`,
//! `containsKey`, a hierarquia (`ListResourceBundle` é um `ResourceBundle`) e a
//! exceção correta para chave ausente.

use rd_testsupport::vm::*;
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const BASE_BUNDLE: &str = "LBundle;";
const PT_BUNDLE: &str = "LBundle_pt;";
const LIST_BUNDLE: &str = "Ljava/util/ListResourceBundle;";

fn bundle_class(b: &mut DexBuilder, name: &str, hello: &str, only: &str) {
    let cls = b.class(name, LIST_BUNDLE);
    let hello_s = b.intern(hello) as u16;
    let only_s = b.intern(only) as u16;
    let key_hello = b.intern("HelloKey") as u16;
    let key_only = b.intern("OnlyKey") as u16;
    let type_rows = b.type_idx("[[Ljava/lang/Object;");
    let type_row = b.type_idx("[Ljava/lang/Object;");

    // <init>()V → super() no-op host (ListResourceBundle modelado).
    let p_void = b.proto_idx("V", vec![]);
    let m_super = b.method_idx(LIST_BUNDLE, p_void, "<init>");
    let mut ctor = op35c(0x70, 1, m_super, [0, 0, 0, 0, 0]); // invoke-direct {p0}
    ctor.extend(op10x(0x0e));
    b.direct(
        cls,
        "<init>",
        "V",
        vec![],
        ACC_PUBLIC,
        Some(b.code(1, 1, 0, ctor)),
    );

    // getContents()[[Ljava/lang/Object;
    let mut code = op11n(0x12, 0, 2); // const/4 v0, 2
    code.extend(op22c(0x23, 0, 0, type_rows)); // new-array v0, v0, Object[][]
    code.extend(op11n(0x12, 1, 2)); // const/4 v1, 2
    code.extend(op22c(0x23, 1, 1, type_row)); // new-array v1, v1, Object[]
    code.extend(op11n(0x12, 2, 0)); // const/4 v2, 0
    code.extend(op21c(0x1a, 3, key_hello)); // const-string v3, "HelloKey"
    code.extend(op23x(0x4d, 3, 1, 2)); // aput-object v3, v1, v2
    code.extend(op11n(0x12, 2, 1)); // const/4 v2, 1
    code.extend(op21c(0x1a, 4, hello_s)); // const-string v4, hello
    code.extend(op23x(0x4d, 4, 1, 2)); // aput-object v4, v1, v2
    code.extend(op11n(0x12, 2, 0)); // const/4 v2, 0
    code.extend(op23x(0x4d, 1, 0, 2)); // aput-object v1, v0, v2
                                       // segunda linha
    code.extend(op11n(0x12, 1, 2)); // const/4 v1, 2
    code.extend(op22c(0x23, 1, 1, type_row)); // new-array v1, v1, Object[]
    code.extend(op11n(0x12, 2, 0)); // const/4 v2, 0
    code.extend(op21c(0x1a, 3, key_only)); // const-string v3, "OnlyKey"
    code.extend(op23x(0x4d, 3, 1, 2)); // aput-object v3, v1, v2
    code.extend(op11n(0x12, 2, 1)); // const/4 v2, 1
    code.extend(op21c(0x1a, 4, only_s)); // const-string v4, only
    code.extend(op23x(0x4d, 4, 1, 2)); // aput-object v4, v1, v2
    code.extend(op11n(0x12, 2, 1)); // const/4 v2, 1
    code.extend(op23x(0x4d, 1, 0, 2)); // aput-object v1, v0, v2
    code.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "getContents",
        "[[Ljava/lang/Object;",
        vec![],
        ACC_PUBLIC,
        Some(b.code(5, 1, 0, code)),
    );
}

fn fixture() -> DexBuilder {
    let mut b = DexBuilder::new();
    bundle_class(&mut b, BASE_BUNDLE, "Hello", "only");
    bundle_class(&mut b, PT_BUNDLE, "Ola", "somente");
    b
}

fn locale(engine: &mut Engine, lang: &str, country: &str) -> ObjRef {
    let l = engine
        .heap
        .alloc_instance("Ljava/util/Locale;".to_string(), Vec::new())
        .expect("locale");
    let lang = engine
        .heap
        .alloc_string(lang.to_string())
        .expect("lang str");
    let country = engine
        .heap
        .alloc_string(country.to_string())
        .expect("country str");
    engine
        .heap
        .put_field(l, "rd_lang", Value::Obj(lang))
        .unwrap();
    engine
        .heap
        .put_field(l, "rd_country", Value::Obj(country))
        .unwrap();
    l
}

fn bundle(engine: &mut Engine, lang: &str, country: &str) -> ObjRef {
    let l = locale(engine, lang, country);
    let base = engine.heap.alloc_string("Bundle".to_string()).unwrap();
    let value = engine
        .invoke_static(
            "Ljava/util/ResourceBundle;",
            "getBundle",
            "(Ljava/lang/String;Ljava/util/Locale;)Ljava/util/ResourceBundle;",
            &[Value::Obj(base), Value::Obj(l)],
        )
        .expect("getBundle");
    match value {
        Value::Obj(r) => r,
        other => panic!("getBundle devolveu {other:?}"),
    }
}

fn get_string(engine: &mut Engine, recv: ObjRef, key: &str) -> Result<Value, rd_vm::VmExit> {
    let key = engine.heap.alloc_string(key.to_string()).unwrap();
    engine.invoke_instance(
        recv,
        "getString",
        "(Ljava/lang/String;)Ljava/lang/String;",
        &[Value::Obj(key)],
    )
}

#[test]
fn get_bundle_prefers_the_locale_variant() {
    let b = fixture();
    let mut engine = engine_of(&b);
    let pt = bundle(&mut engine, "pt", "BR");
    assert_eq!(
        engine.heap.class_of(pt).unwrap(),
        PT_BUNDLE,
        "pt-BR precisa resolver LBundle_pt; (variante de idioma)"
    );
    let en = bundle(&mut engine, "en", "US");
    assert_eq!(
        engine.heap.class_of(en).unwrap(),
        BASE_BUNDLE,
        "sem variante en o candidato passa a ser a classe base"
    );
}

#[test]
fn list_resource_bundle_is_a_resource_bundle() {
    let b = fixture();
    let engine = engine_of(&b);
    assert!(engine
        .cp
        .is_subtype(PT_BUNDLE, "Ljava/util/ResourceBundle;"));
    assert!(engine
        .cp
        .is_subtype(PT_BUNDLE, "Ljava/util/ListResourceBundle;"));
}

#[test]
fn get_string_reads_the_located_pairs() {
    let b = fixture();
    let mut engine = engine_of(&b);
    let pt = bundle(&mut engine, "pt", "BR");
    let value = get_string(&mut engine, pt, "HelloKey").expect("HelloKey existe");
    let Value::Obj(s) = value else {
        panic!("getString devolveu {value:?}");
    };
    assert_eq!(engine.heap.as_str(s).unwrap(), "Ola");

    let value = get_string(&mut engine, pt, "OnlyKey").expect("OnlyKey existe");
    let Value::Obj(s) = value else {
        panic!("getString devolveu {value:?}");
    };
    assert_eq!(engine.heap.as_str(s).unwrap(), "somente");
}

#[test]
fn contains_key_and_handle_get_object_do_not_throw() {
    let b = fixture();
    let mut engine = engine_of(&b);
    let pt = bundle(&mut engine, "pt", "BR");
    let key = engine.heap.alloc_string("HelloKey".to_string()).unwrap();
    let present = engine
        .invoke_instance(
            pt,
            "containsKey",
            "(Ljava/lang/String;)Z",
            &[Value::Obj(key)],
        )
        .expect("containsKey");
    assert_eq!(present, Value::Int(1));

    let missing = engine.heap.alloc_string("Nope".to_string()).unwrap();
    let absent = engine
        .invoke_instance(
            pt,
            "containsKey",
            "(Ljava/lang/String;)Z",
            &[Value::Obj(missing)],
        )
        .expect("containsKey");
    assert_eq!(absent, Value::Int(0));

    let handled = engine
        .invoke_instance(
            pt,
            "handleGetObject",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            &[Value::Obj(missing)],
        )
        .expect("handleGetObject devolve null, não lança");
    assert_eq!(handled, Value::Null);
}

#[test]
fn a_missing_key_throws_missing_resource_exception() {
    let b = fixture();
    let mut engine = engine_of(&b);
    let pt = bundle(&mut engine, "pt", "BR");
    let failure = get_string(&mut engine, pt, "Absent").expect_err("chave ausente lança");
    let message = format!("{failure:?}");
    assert!(
        message.contains("MissingResourceException"),
        "esperava MissingResourceException, veio: {message}"
    );
}
