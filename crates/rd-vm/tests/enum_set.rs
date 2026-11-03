//! `java.util.EnumSet.of(...)` — a fábrica que APKs reais usam para conjuntos de
//! tamanho fixo (visto no NewPipe dentro de `App.onCreate`).

mod common;

use common::*;
use rd_vm::{Engine, Value};

fn set_of(e: &mut Engine, sig: &str, args: &[Value]) -> Result<Value, rd_vm::VmExit> {
    e.invoke_static("Ljava/util/EnumSet;", "of", sig, args)
}

fn element(e: &mut Engine, name: &str) -> Value {
    Value::Obj(
        e.heap
            .alloc_instance(format!("LElem{name};"), Vec::new())
            .unwrap(),
    )
}

#[test]
fn of_builds_a_set_with_the_given_elements() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let a = element(&mut e, "A");
    let c = element(&mut e, "C");
    let set = set_of(
        &mut e,
        "(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;",
        &[a.clone(), c.clone()],
    )
    .expect("EnumSet.of(a, c)");
    let Value::Obj(r) = set else {
        panic!("EnumSet.of devolveu {set:?}");
    };
    assert_eq!(e.heap.class_of(r).unwrap(), "Ljava/util/EnumSet;");
    let elems = match e.fw.objects.get(&r) {
        Some(rd_vm::framework::HostObj::List { elems }) => elems.clone(),
        other => panic!("esperava Set host, veio {other:?}"),
    };
    assert_eq!(elems, vec![a, c], "EnumSet.of preserva a ordem declarada");
}

#[test]
fn of_rejects_duplicates_and_null() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let a = element(&mut e, "A");
    let duplicate = set_of(
        &mut e,
        "(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;",
        &[a.clone(), a],
    );
    let message = format!("{duplicate:?}");
    assert!(
        message.contains("IllegalArgumentException"),
        "duplicata precisa ser IAE, veio: {message}"
    );

    let null = set_of(
        &mut e,
        "(Ljava/lang/Enum;)Ljava/util/EnumSet;",
        &[Value::Int(0)],
    );
    let message = format!("{null:?}");
    assert!(
        message.contains("NullPointerException"),
        "null precisa ser NPE, veio: {message}"
    );
}

#[test]
fn an_unknown_arity_stays_a_typed_gap() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let result = set_of(
        &mut e,
        "(Ljava/util/Collection;)Ljava/util/EnumSet;",
        &[Value::Null],
    );
    assert_eq!(
        result.ok(),
        None,
        "copyOf ainda não é modelado: o despacho devolve None (gap), não um set vazio"
    );
}
