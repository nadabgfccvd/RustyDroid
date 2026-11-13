//! `SoundPool.load*` queues an `OnLoadCompleteListener.onLoadComplete` that the
//! main Looper delivers (`drain_looper`). The listener is read at delivery
//! time, the queue keeps its pool alive across GC, and a pool with no listener
//! drops its completion without failing.

#[macro_use]
mod common;
use common::*;
use rd_vm::value::Value;

const LISTENER: &str = "LSoundListener;";
const POOL_LISTENER_SIG: &str = "(Landroid/media/SoundPool$OnLoadCompleteListener;)V";
const LOAD_SIG: &str = "(Landroid/content/Context;II)I";

/// DEX listener: `onLoadComplete(pool, sampleId, status)` stores `sampleId`
/// in its own `last` field, so the test can observe the delivery.
fn engine_with_listener() -> rd_vm::Engine {
    let mut b = DexBuilder::new();
    let cls = b.class(LISTENER, "Ljava/lang/Object;");
    b.instance_field(cls, "last", "I");
    let last = b.field_idx(LISTENER, "I", "last");
    let mut code = op22c(0x5b, 2, 0, last); // iput v2 (sampleId), v0 (this), last
    code.extend(op10x(0x0E)); // return-void
    b.r#virtual(
        cls,
        "onLoadComplete",
        "V",
        vec!["Landroid/media/SoundPool;", "I", "I"],
        ACC_PUBLIC,
        Some(b.code(4, 4, 0, code)),
    );
    engine_of(&b)
}

fn new_pool(e: &mut rd_vm::Engine) -> rd_vm::heap::ObjRef {
    let pool = e
        .heap
        .alloc_instance("Landroid/media/SoundPool;".into(), vec![])
        .unwrap();
    e.attach_host_state("Landroid/media/SoundPool;", pool)
        .unwrap();
    e.invoke_instance(
        pool,
        "<init>",
        "(III)V",
        &[Value::Int(4), Value::Int(3), Value::Int(0)],
    )
    .unwrap();
    pool
}

fn load(e: &mut rd_vm::Engine, pool: rd_vm::heap::ObjRef) -> Value {
    e.invoke_instance(
        pool,
        "load",
        LOAD_SIG,
        &[Value::Null, Value::Int(1), Value::Int(0)],
    )
    .unwrap()
}

fn set_listener(e: &mut rd_vm::Engine, pool: rd_vm::heap::ObjRef, listener: rd_vm::heap::ObjRef) {
    e.invoke_instance(
        pool,
        "setOnLoadCompleteListener",
        POOL_LISTENER_SIG,
        &[Value::Obj(listener)],
    )
    .unwrap();
}

fn new_listener(e: &mut rd_vm::Engine) -> rd_vm::heap::ObjRef {
    let l = e.heap.alloc_instance(LISTENER.into(), vec![]).unwrap();
    e.heap.put_field(l, "last", Value::Int(0)).unwrap();
    l
}

fn last_delivered(e: &rd_vm::Engine, l: rd_vm::heap::ObjRef) -> Value {
    e.heap.get_field(l, "last").unwrap()
}

#[test]
fn completion_is_delivered_to_listener_on_drain() {
    let mut e = engine_with_listener();
    let pool = new_pool(&mut e);
    let listener = new_listener(&mut e);
    set_listener(&mut e, pool, listener);
    let id = load(&mut e, pool);
    assert_eq!(id, Value::Int(1));
    assert_eq!(
        last_delivered(&e, listener),
        Value::Int(0),
        "queued, not yet delivered"
    );

    e.drain_looper().unwrap();
    assert_eq!(
        last_delivered(&e, listener),
        Value::Int(1),
        "delivered with the sound id"
    );
    assert_eq!(
        e.drain_looper().unwrap(),
        0,
        "queue is empty after the drain"
    );
}

#[test]
fn listener_set_after_load_still_receives_the_callback() {
    let mut e = engine_with_listener();
    let pool = new_pool(&mut e);
    let listener = new_listener(&mut e);
    load(&mut e, pool);
    set_listener(&mut e, pool, listener);
    e.drain_looper().unwrap();
    assert_eq!(last_delivered(&e, listener), Value::Int(1));
}

#[test]
fn pool_without_listener_drops_its_completion_quietly() {
    let mut e = engine_with_listener();
    let pool = new_pool(&mut e);
    load(&mut e, pool);
    e.drain_looper().unwrap();

    // A later listener only sees completions queued after it was set.
    let listener = new_listener(&mut e);
    set_listener(&mut e, pool, listener);
    assert_eq!(load(&mut e, pool), Value::Int(2));
    e.drain_looper().unwrap();
    assert_eq!(last_delivered(&e, listener), Value::Int(2));
}

#[test]
fn queued_completion_keeps_its_pool_and_listener_alive_across_gc() {
    // GC root scanning is conservative: any integer in the state dump that
    // equals an ObjRef pins that object. Small ids (1, 2) collide with ordinary
    // ints such as `Int(1)`, so the pool and listener get high ids here, after
    // a block of garbage that the collection is expected to free.
    let mut e = engine_with_listener();
    for _ in 0..300 {
        e.heap
            .alloc_instance("Ljava/lang/Object;".into(), vec![])
            .unwrap();
    }
    // No host state attached: otherwise `fw.objects` roots the pool.
    let pool = e
        .heap
        .alloc_instance("Landroid/media/SoundPool;".into(), vec![])
        .unwrap();
    e.invoke_instance(
        pool,
        "<init>",
        "(III)V",
        &[Value::Int(4), Value::Int(3), Value::Int(0)],
    )
    .unwrap();
    let listener = new_listener(&mut e);
    set_listener(&mut e, pool, listener);
    load(&mut e, pool);

    // Nothing but the queue references the pool or the listener (its only
    // path is the pool's field); the test's locals are not roots.
    e.gc();
    e.drain_looper().unwrap();
    assert_eq!(last_delivered(&e, listener), Value::Int(1));
}
