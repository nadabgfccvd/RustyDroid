//! `Collections.binarySearch` over `List` (with and without a `Comparator`).
//!
//! Gap hit by the real NewPipe launch: `App.onCreate` →
//! `PreferenceManager.setDefaultValues` sorts and searches preference keys with
//! `Collections.binarySearch(List, Object)`, which the intrinsic surface did
//! not implement, so the whole chain stopped with `NOT_IMPLEMENTED` before any
//! UI existed.
//!
//! JDK contract proven here: found → index; absent → `-(insertion point) - 1`;
//! null list → NPE (not a silent 0).

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

fn search(e: &mut Engine, list: Value, key: Value) -> Value {
    rd_vm::intrinsics::call_static_intrinsic(
        e,
        COLLECTIONS,
        "binarySearch",
        "(Ljava/util/List;Ljava/lang/Object;)I",
        &[list, key],
    )
    .expect("binarySearch runs")
    .expect("intrinsic answers")
}

#[test]
fn finds_existing_key_and_reports_insertion_point_for_a_missing_one() {
    let mut e = engine();
    let list = list_of(&mut e, &[Value::Int(1), Value::Int(3), Value::Int(5)]);
    assert_eq!(search(&mut e, list.clone(), Value::Int(1)), Value::Int(0));
    assert_eq!(search(&mut e, list.clone(), Value::Int(5)), Value::Int(2));
    // 4 would sit at index 2 → -(2) - 1 = -3 (JDK encoding, not "not found").
    assert_eq!(search(&mut e, list.clone(), Value::Int(4)), Value::Int(-3));
    // Smaller than every element → -(0) - 1 = -1.
    assert_eq!(search(&mut e, list.clone(), Value::Int(0)), Value::Int(-1));
    // Greater than every element → -(3) - 1 = -4.
    assert_eq!(search(&mut e, list, Value::Int(9)), Value::Int(-4));
}

#[test]
fn empty_list_answers_minus_one() {
    let mut e = engine();
    let list = list_of(&mut e, &[]);
    assert_eq!(search(&mut e, list, Value::Int(7)), Value::Int(-1));
}

#[test]
fn string_keys_use_compare_to_on_the_guest_string() {
    let mut e = engine();
    let a = e.heap.alloc_string("alpha".to_string()).unwrap();
    let b = e.heap.alloc_string("beta".to_string()).unwrap();
    let c = e.heap.alloc_string("gamma".to_string()).unwrap();
    let list = list_of(&mut e, &[Value::Obj(a), Value::Obj(b), Value::Obj(c)]);
    let beta = e.heap.alloc_string("beta".to_string()).unwrap();
    assert_eq!(
        search(&mut e, list.clone(), Value::Obj(beta)),
        Value::Int(1)
    );
    let delta = e.heap.alloc_string("delta".to_string()).unwrap();
    // "delta" sorts between "beta" (1) and "gamma" (2) → -(2) - 1.
    assert_eq!(search(&mut e, list, Value::Obj(delta)), Value::Int(-3));
}

#[test]
fn null_list_is_a_null_pointer_exception() {
    let mut e = engine();
    match rd_vm::intrinsics::call_static_intrinsic(
        &mut e,
        COLLECTIONS,
        "binarySearch",
        "(Ljava/util/List;Ljava/lang/Object;)I",
        &[Value::Null, Value::Int(1)],
    ) {
        Err(VmExit::Exception(t)) => assert_eq!(t.class, "Ljava/lang/NullPointerException;"),
        other => panic!("esperava NPE, veio {other:?}"),
    }
}
