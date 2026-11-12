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

fn set_elems(e: &Engine, set: &Value) -> Vec<Value> {
    let Value::Obj(r) = set else {
        panic!("esperava EnumSet, veio {set:?}");
    };
    match e.fw.objects.get(r) {
        Some(rd_vm::framework::HostObj::List { elems }) => elems.clone(),
        other => panic!("esperava Set host, veio {other:?}"),
    }
}

fn class_obj(e: &mut Engine, desc: &str) -> Value {
    let r = e
        .heap
        .alloc_instance("Ljava/lang/Class;".to_string(), Vec::new())
        .unwrap();
    let s = e.heap.alloc_string(desc.to_string()).unwrap();
    e.heap.put_field(r, "class_name", Value::Obj(s)).unwrap();
    Value::Obj(r)
}

#[test]
fn none_of_returns_an_empty_set() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let cls = class_obj(&mut e, "LElem;");
    let set = e
        .invoke_static(
            "Ljava/util/EnumSet;",
            "noneOf",
            "(Ljava/lang/Class;)Ljava/util/EnumSet;",
            &[cls],
        )
        .expect("EnumSet.noneOf");
    assert!(set_elems(&e, &set).is_empty(), "noneOf é vazio");
}

#[test]
fn none_of_null_class_is_npe() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let result = e.invoke_static(
        "Ljava/util/EnumSet;",
        "noneOf",
        "(Ljava/lang/Class;)Ljava/util/EnumSet;",
        &[Value::Null],
    );
    assert!(
        format!("{result:?}").contains("NullPointerException"),
        "noneOf(null) precisa ser NPE, veio: {result:?}"
    );
}

#[test]
fn copy_of_set_is_an_independent_copy() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let a = element(&mut e, "A");
    let set = set_of(
        &mut e,
        "(Ljava/lang/Enum;)Ljava/util/EnumSet;",
        std::slice::from_ref(&a),
    )
    .expect("EnumSet.of(a)");
    let copy = e
        .invoke_static(
            "Ljava/util/EnumSet;",
            "copyOf",
            "(Ljava/util/EnumSet;)Ljava/util/EnumSet;",
            std::slice::from_ref(&set),
        )
        .expect("EnumSet.copyOf(set)");
    assert_eq!(set_elems(&e, &copy), vec![a]);
    assert_ne!(set, copy, "copyOf devolve um conjunto novo");
}

#[test]
fn of_varargs_collects_first_and_rest() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let a = element(&mut e, "A");
    let c = element(&mut e, "C");
    let d = element(&mut e, "D");
    // Array de Enum com [c, d] como "rest".
    let arr = e
        .heap
        .alloc_array(rd_vm::heap::ElemKind::Obj, 2, "Ljava/lang/Enum;")
        .unwrap();
    e.heap
        .array_elems_mut(arr)
        .unwrap()
        .extend([c.clone(), d.clone()]);
    let set = set_of(
        &mut e,
        "(Ljava/lang/Enum;[Ljava/lang/Enum;)Ljava/util/EnumSet;",
        &[a.clone(), Value::Obj(arr)],
    )
    .expect("EnumSet.of(a, c, d) varargs");
    assert_eq!(set_elems(&e, &set), vec![a, c, d]);
}

#[test]
fn of_varargs_rejects_duplicates() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let a = element(&mut e, "A");
    let arr = e
        .heap
        .alloc_array(rd_vm::heap::ElemKind::Obj, 1, "Ljava/lang/Enum;")
        .unwrap();
    e.heap.array_elems_mut(arr).unwrap().extend([a.clone()]);
    let result = set_of(
        &mut e,
        "(Ljava/lang/Enum;[Ljava/lang/Enum;)Ljava/util/EnumSet;",
        &[a, Value::Obj(arr)],
    );
    assert!(
        format!("{result:?}").contains("IllegalArgumentException"),
        "duplicata em varargs precisa ser IAE, veio: {result:?}"
    );
}

/// Enum DEX real com 3 constantes (A, B, C) para allOf/range/complementOf.
/// As constantes são injetadas em `statics` após `ensure_initialized`
/// (o `<clinit>` sintético não existe; o que se testa é a enumeração).
fn dex_enum_engine() -> (Engine, Value, Value, Value) {
    let mut b = DexBuilder::new();
    let cls = b.class("LElem;", "Ljava/lang/Enum;");
    b.static_field(cls, "A", "LElem;", SVal::Int(0));
    b.static_field(cls, "B", "LElem;", SVal::Int(0));
    b.static_field(cls, "C", "LElem;", SVal::Int(0));
    b.field_idx("LElem;", "LElem;", "A");
    b.field_idx("LElem;", "LElem;", "B");
    b.field_idx("LElem;", "LElem;", "C");
    let mut e = engine_of(&b);
    e.ensure_initialized("LElem;").expect("clinit LElem");
    let a = Value::Obj(
        e.heap
            .alloc_instance("LElem;".to_string(), Vec::new())
            .unwrap(),
    );
    let bb = Value::Obj(
        e.heap
            .alloc_instance("LElem;".to_string(), Vec::new())
            .unwrap(),
    );
    let c = Value::Obj(
        e.heap
            .alloc_instance("LElem;".to_string(), Vec::new())
            .unwrap(),
    );
    e.statics
        .insert(("LElem;".to_string(), "A".to_string()), a.clone());
    e.statics
        .insert(("LElem;".to_string(), "B".to_string()), bb.clone());
    e.statics
        .insert(("LElem;".to_string(), "C".to_string()), c.clone());
    (e, a, bb, c)
}

#[test]
fn all_of_returns_all_constants_in_declaration_order() {
    let (mut e, a, bb, c) = dex_enum_engine();
    let cls = class_obj(&mut e, "LElem;");
    let set = e
        .invoke_static(
            "Ljava/util/EnumSet;",
            "allOf",
            "(Ljava/lang/Class;)Ljava/util/EnumSet;",
            &[cls],
        )
        .expect("EnumSet.allOf");
    assert_eq!(set_elems(&e, &set), vec![a, bb, c]);
}

#[test]
fn range_returns_the_inclusive_slice() {
    let (mut e, a, bb, c) = dex_enum_engine();
    let set = e
        .invoke_static(
            "Ljava/util/EnumSet;",
            "range",
            "(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;",
            &[a.clone(), bb.clone()],
        )
        .expect("EnumSet.range(A, B)");
    assert_eq!(set_elems(&e, &set), vec![a.clone(), bb]);
    let full = e
        .invoke_static(
            "Ljava/util/EnumSet;",
            "range",
            "(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;",
            &[a, c],
        )
        .expect("EnumSet.range(A, C)");
    assert_eq!(set_elems(&e, &full).len(), 3);
}

#[test]
fn range_reversed_is_iae() {
    let (mut e, _a, bb, c) = dex_enum_engine();
    let result = e.invoke_static(
        "Ljava/util/EnumSet;",
        "range",
        "(Ljava/lang/Enum;Ljava/lang/Enum;)Ljava/util/EnumSet;",
        &[c, bb],
    );
    assert!(
        format!("{result:?}").contains("IllegalArgumentException"),
        "range(C, B) precisa ser IAE, veio: {result:?}"
    );
}

#[test]
fn complement_of_drops_the_present_elements() {
    let (mut e, a, bb, c) = dex_enum_engine();
    let set = set_of(
        &mut e,
        "(Ljava/lang/Enum;)Ljava/util/EnumSet;",
        std::slice::from_ref(&bb),
    )
    .expect("EnumSet.of(B)");
    let comp = e
        .invoke_static(
            "Ljava/util/EnumSet;",
            "complementOf",
            "(Ljava/util/EnumSet;)Ljava/util/EnumSet;",
            &[set],
        )
        .expect("EnumSet.complementOf");
    assert_eq!(set_elems(&e, &comp), vec![a, c]);
}

#[test]
fn all_of_unknown_class_stays_a_typed_gap() {
    let b = DexBuilder::new();
    let mut e = engine_of(&b);
    let cls = class_obj(&mut e, "LNoSuchEnum;");
    let result = e.invoke_static(
        "Ljava/util/EnumSet;",
        "allOf",
        "(Ljava/lang/Class;)Ljava/util/EnumSet;",
        &[cls],
    );
    assert!(
        result.as_ref().ok().is_none(),
        "enum fora do DEX não é inventado: gap tipado, veio: {result:?}"
    );
}
