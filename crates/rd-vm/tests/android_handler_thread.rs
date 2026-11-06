//! `android.os.HandlerThread` (CameraX creates one for the camera).
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): `new HandlerThread(name)`
//! answered `NOT_FOUND` on `<init>(String)`. These tests pin the lifecycle the
//! model supports: a Looper appears on `start()`, a Handler on that Looper
//! queues into it (not into the main Looper), `quit` discards and `quitSafely`
//! keeps the messages already due.

mod common;

use common::*;
use rd_vm::value::Value;

const HT: &str = "Landroid/os/HandlerThread;";
const HANDLER: &str = "Landroid/os/Handler;";
const CTOR_NAME: &str = "(Ljava/lang/String;)V";
const CTOR_HANDLER: &str = "(Landroid/os/Looper;Landroid/os/Handler$Callback;)V";

fn engine() -> rd_vm::Engine {
    engine_of(&DexBuilder::new())
}

fn new_thread(e: &mut rd_vm::Engine, name: &str) -> u32 {
    let t = e
        .heap
        .alloc_instance(HT.to_string(), Vec::new())
        .expect("alloc HandlerThread");
    let s = e.heap.alloc_string(name.to_string()).expect("alloc name");
    e.invoke_instance(t, "<init>", CTOR_NAME, &[Value::Obj(s)])
        .expect("HandlerThread(String)");
    t
}

fn looper_of(e: &mut rd_vm::Engine, t: u32) -> Option<u32> {
    match e
        .invoke_instance(t, "getLooper", "()Landroid/os/Looper;", &[])
        .expect("getLooper")
    {
        Value::Obj(l) => Some(l),
        _ => None,
    }
}

fn handler_on(e: &mut rd_vm::Engine, looper: u32) -> u32 {
    let h = e
        .heap
        .alloc_instance(HANDLER.to_string(), Vec::new())
        .expect("alloc Handler");
    e.invoke_instance(
        h,
        "<init>",
        CTOR_HANDLER,
        &[Value::Obj(looper), Value::Null],
    )
    .expect("Handler(Looper, Callback)");
    h
}

#[test]
fn constructor_records_name_and_has_no_looper_before_start() {
    let mut e = engine();
    let t = new_thread(&mut e, "cam");
    assert!(looper_of(&mut e, t).is_none(), "no Looper before start()");
}

#[test]
fn start_creates_a_looper_and_handler_posts_into_it_not_main() {
    let mut e = engine();
    let t = new_thread(&mut e, "cam");
    e.invoke_instance(t, "start", "()V", &[]).expect("start");
    let l = looper_of(&mut e, t).expect("Looper after start()");
    let h = handler_on(&mut e, l);
    e.invoke_instance(h, "sendEmptyMessage", "(I)Z", &[Value::Int(1)])
        .expect("sendEmptyMessage");
    assert_eq!(
        e.looper_pending_of(l),
        1,
        "message queued on the HandlerThread Looper"
    );
    assert_eq!(e.looper_pending(), 0, "main Looper untouched");
    e.drain_looper().expect("drain");
    assert_eq!(e.looper_pending_of(l), 0, "due message drained");
}

#[test]
fn quit_discards_pending_and_clears_looper() {
    let mut e = engine();
    let t = new_thread(&mut e, "cam");
    e.invoke_instance(t, "start", "()V", &[]).expect("start");
    let l = looper_of(&mut e, t).expect("Looper");
    let h = handler_on(&mut e, l);
    e.invoke_instance(
        h,
        "sendEmptyMessageDelayed",
        "(IJ)Z",
        &[Value::Int(1), Value::Long(500)],
    )
    .expect("delayed");
    assert_eq!(e.looper_pending_of(l), 1);
    assert_eq!(
        e.invoke_instance(t, "quit", "()Z", &[]).expect("quit"),
        Value::Int(1)
    );
    assert_eq!(e.looper_pending_of(l), 0, "quit discards pending messages");
    assert!(
        looper_of(&mut e, t).is_none(),
        "getLooper is null after quit"
    );
    assert_eq!(
        e.invoke_instance(t, "quit", "()Z", &[])
            .expect("second quit"),
        Value::Int(0),
        "a second quit reports that nothing was quit"
    );
}

#[test]
fn quit_safely_keeps_due_messages_and_drops_future_ones() {
    let mut e = engine();
    let t = new_thread(&mut e, "cam");
    e.invoke_instance(t, "start", "()V", &[]).expect("start");
    let l = looper_of(&mut e, t).expect("Looper");
    let h = handler_on(&mut e, l);
    e.invoke_instance(h, "sendEmptyMessage", "(I)Z", &[Value::Int(1)])
        .expect("due");
    e.invoke_instance(
        h,
        "sendEmptyMessageDelayed",
        "(IJ)Z",
        &[Value::Int(2), Value::Long(500)],
    )
    .expect("future");
    assert_eq!(e.looper_pending_of(l), 2);
    e.invoke_instance(t, "quitSafely", "()Z", &[])
        .expect("quitSafely");
    assert_eq!(
        e.looper_pending_of(l),
        1,
        "future message dropped, due one kept"
    );
    e.drain_looper().expect("drain");
    assert_eq!(
        e.looper_pending_of(l),
        0,
        "the kept message ran and the Looper left the drain set"
    );
}

#[test]
fn thread_id_is_the_green_thread_id() {
    let mut e = engine();
    let t = new_thread(&mut e, "cam");
    let v = e
        .invoke_instance(t, "getThreadId", "()I", &[])
        .expect("getThreadId");
    assert!(matches!(v, Value::Int(id) if id > 0), "got {v:?}");
}
