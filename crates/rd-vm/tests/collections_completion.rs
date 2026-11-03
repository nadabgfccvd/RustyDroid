//! `java.util.Collections` statics that real apps call all the time, plus the
//! `reverseOrder()` comparator that used to be a `NOT_IMPLEMENTED` gap: sorting
//! descending without a lambda or a hand-written `Comparator` class.
//!
//! Every case asserts the JDK answer (including the exception classes for the
//! boundary conditions), because a wrong-but-plausible value here silently
//! corrupts app state instead of failing loudly.

use rd_vm::err::VmExit;
use rd_vm::value::Value;
use rd_vm::{Engine, VmConfig};

const COLLECTIONS: &str = "Ljava/util/Collections;";

fn engine() -> Engine {
    Engine::new(vec![], VmConfig::default())
}

fn list_of(e: &mut Engine, values: &[Value]) -> Value {
    let r = e
        .heap
        .alloc_instance("Ljava/util/ArrayList;".to_string(), vec![])
        .unwrap();
    e.invoke_instance(r, "<init>", "()V", &[]).unwrap();
    for v in values {
        e.invoke_instance(r, "add", "(Ljava/lang/Object;)Z", std::slice::from_ref(v))
            .unwrap();
    }
    Value::Obj(r)
}

fn elems(e: &mut Engine, list: &Value) -> Vec<Value> {
    let Value::Obj(r) = list else { panic!("lista") };
    match e.fw.objects.get(r) {
        Some(rd_vm::framework::HostObj::List { elems }) => elems.clone(),
        other => panic!("sem estado host: {other:?}"),
    }
}

fn call(e: &mut Engine, name: &str, sig: &str, args: &[Value]) -> Option<Value> {
    rd_vm::intrinsics::call_static_intrinsic(e, COLLECTIONS, name, sig, args).unwrap()
}

#[test]
fn frequency_counts_with_equals_semantics() {
    let mut e = engine();
    let list = list_of(&mut e, &[Value::Int(1), Value::Int(2), Value::Int(1)]);
    assert_eq!(
        call(
            &mut e,
            "frequency",
            "(Ljava/util/Collection;Ljava/lang/Object;)I",
            &[list, Value::Int(1)]
        ),
        Some(Value::Int(2))
    );
}

#[test]
fn n_copies_is_an_immutable_list_of_n_references() {
    let mut e = engine();
    let s = e.heap.alloc_string("x".to_string()).unwrap();
    let copies = call(
        &mut e,
        "nCopies",
        "(ILjava/lang/Object;)Ljava/util/List;",
        &[Value::Int(3), Value::Obj(s)],
    )
    .expect("nCopies responde");
    assert_eq!(elems(&mut e, &copies).len(), 3);
    // Immutable marker class: mutation is UnsupportedOperationException.
    let Value::Obj(r) = copies else { panic!() };
    match e.invoke_instance(r, "add", "(Ljava/lang/Object;)Z", &[Value::Int(9)]) {
        Err(VmExit::Exception(t)) => assert_eq!(
            t.class, "Ljava/lang/UnsupportedOperationException;",
            "nCopies precisa ser imutável como no JDK"
        ),
        other => panic!("esperava UOE, veio {other:?}"),
    }
}

#[test]
fn negative_n_copies_is_an_illegal_argument() {
    let mut e = engine();
    match rd_vm::intrinsics::call_static_intrinsic(
        &mut e,
        COLLECTIONS,
        "nCopies",
        "(ILjava/lang/Object;)Ljava/util/List;",
        &[Value::Int(-1), Value::Null],
    ) {
        Err(VmExit::Exception(t)) => assert_eq!(t.class, "Ljava/lang/IllegalArgumentException;"),
        other => panic!("esperava IAE, veio {other:?}"),
    }
}

#[test]
fn fill_swap_rotate_and_copy_edit_the_host_list_in_place() {
    let mut e = engine();
    let list = list_of(&mut e, &[Value::Int(1), Value::Int(2), Value::Int(3)]);
    call(
        &mut e,
        "swap",
        "(Ljava/util/List;II)V",
        &[list.clone(), Value::Int(0), Value::Int(2)],
    );
    assert_eq!(
        elems(&mut e, &list),
        vec![Value::Int(3), Value::Int(2), Value::Int(1)]
    );
    // rotate(1) moves every element one position to the right (tail wraps).
    call(
        &mut e,
        "rotate",
        "(Ljava/util/List;I)V",
        &[list.clone(), Value::Int(1)],
    );
    assert_eq!(
        elems(&mut e, &list),
        vec![Value::Int(1), Value::Int(3), Value::Int(2)]
    );
    let src = list_of(&mut e, &[Value::Int(7), Value::Int(8)]);
    call(
        &mut e,
        "copy",
        "(Ljava/util/List;Ljava/util/List;)V",
        &[list.clone(), src],
    );
    assert_eq!(
        elems(&mut e, &list),
        vec![Value::Int(7), Value::Int(8), Value::Int(2)]
    );
    call(
        &mut e,
        "fill",
        "(Ljava/util/List;Ljava/lang/Object;)V",
        &[list.clone(), Value::Int(0)],
    );
    assert_eq!(
        elems(&mut e, &list),
        vec![Value::Int(0), Value::Int(0), Value::Int(0)]
    );
}

#[test]
fn swap_out_of_bounds_is_an_index_out_of_bounds_exception() {
    let mut e = engine();
    let list = list_of(&mut e, &[Value::Int(1)]);
    match rd_vm::intrinsics::call_static_intrinsic(
        &mut e,
        COLLECTIONS,
        "swap",
        "(Ljava/util/List;II)V",
        &[list, Value::Int(0), Value::Int(5)],
    ) {
        Err(VmExit::Exception(t)) => assert_eq!(t.class, "Ljava/lang/IndexOutOfBoundsException;"),
        other => panic!("esperava IOOBE, veio {other:?}"),
    }
}

#[test]
fn copy_into_a_smaller_destination_is_an_index_out_of_bounds_exception() {
    let mut e = engine();
    let dest = list_of(&mut e, &[Value::Int(0)]);
    let src = list_of(&mut e, &[Value::Int(1), Value::Int(2)]);
    match rd_vm::intrinsics::call_static_intrinsic(
        &mut e,
        COLLECTIONS,
        "copy",
        "(Ljava/util/List;Ljava/util/List;)V",
        &[dest, src],
    ) {
        Err(VmExit::Exception(t)) => assert_eq!(t.class, "Ljava/lang/IndexOutOfBoundsException;"),
        other => panic!("esperava IOOBE, veio {other:?}"),
    }
}

#[test]
fn replace_all_reports_whether_anything_changed() {
    let mut e = engine();
    let list = list_of(&mut e, &[Value::Int(1), Value::Int(2), Value::Int(1)]);
    let sig = "(Ljava/util/List;Ljava/lang/Object;Ljava/lang/Object;)Z";
    assert_eq!(
        call(
            &mut e,
            "replaceAll",
            sig,
            &[list.clone(), Value::Int(1), Value::Int(5)]
        ),
        Some(Value::Int(1))
    );
    assert_eq!(
        elems(&mut e, &list),
        vec![Value::Int(5), Value::Int(2), Value::Int(5)]
    );
    assert_eq!(
        call(
            &mut e,
            "replaceAll",
            sig,
            &[list, Value::Int(42), Value::Int(5)]
        ),
        Some(Value::Int(0))
    );
}

#[test]
fn disjoint_compares_elements_not_identities() {
    let mut e = engine();
    let a = list_of(&mut e, &[Value::Int(1), Value::Int(2)]);
    let b = list_of(&mut e, &[Value::Int(3), Value::Int(4)]);
    let c = list_of(&mut e, &[Value::Int(2), Value::Int(9)]);
    let sig = "(Ljava/util/Collection;Ljava/util/Collection;)Z";
    assert_eq!(
        call(&mut e, "disjoint", sig, &[a.clone(), b]),
        Some(Value::Int(1))
    );
    assert_eq!(call(&mut e, "disjoint", sig, &[a, c]), Some(Value::Int(0)));
}

#[test]
fn reverse_order_sorts_descending_and_is_a_singleton() {
    let mut e = engine();
    let cmp = call(&mut e, "reverseOrder", "()Ljava/util/Comparator;", &[])
        .expect("reverseOrder responde");
    let again = call(&mut e, "reverseOrder", "()Ljava/util/Comparator;", &[]).unwrap();
    assert_eq!(cmp, again, "JDK: reverseOrder() devolve o mesmo singleton");
    let Value::Obj(r) = cmp else { panic!() };
    // natural order says 1 < 2; reversed says 1 > 2.
    let out = e
        .invoke_instance(
            r,
            "compare",
            "(Ljava/lang/Object;Ljava/lang/Object;)I",
            &[Value::Int(1), Value::Int(2)],
        )
        .expect("compare do reverseOrder");
    assert!(out.as_int().unwrap() > 0, "reverseOrder compare invertido");
    // It is usable as the comparator argument of Collections.sort.
    let list = list_of(&mut e, &[Value::Int(1), Value::Int(3), Value::Int(2)]);
    call(
        &mut e,
        "sort",
        "(Ljava/util/List;Ljava/util/Comparator;)V",
        &[list.clone(), Value::Obj(r)],
    );
    assert_eq!(
        elems(&mut e, &list),
        vec![Value::Int(3), Value::Int(2), Value::Int(1)]
    );
}
