//! `android.os.HandlerThread`: a thread with its own Looper and message queue.
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): CameraX creates a
//! `HandlerThread` for the camera and answered `NOT_FOUND` on its constructor.
//!
//! Model:
//! - `HandlerThread` is a `java.lang.Thread` subclass. The constructor records
//!   the name; the priority argument is accepted and not applied (no scheduler).
//! - `start()` creates a Looper with its own queue (`HostObj::Looper`). The
//!   queue is the same kind the main Looper has, so `new Handler(looper)` posts
//!   into it through `handler_msg.rs` with no other change.
//! - Messages in a started HandlerThread's Looper are drained by
//!   `Engine::drain_looper`, in the same virtual-time deadline order as the main
//!   Looper. The thread's own `run()` is empty (the body is the drain, not a
//!   `Looper.loop()` green thread). This is a documented deviation: the VM has
//!   one deterministic consumer, so two queues never run in parallel.
//! - `quit()` discards pending messages and returns `true` when the Looper was
//!   active. `quitSafely()` discards only the messages not yet due; the due ones
//!   still run on the next drain. `getLooper()` returns `null` after either call.
//! - `getThreadId()` returns the green thread id.

use crate::engine::Engine;
use crate::err::VmExit;
use crate::framework::HostObj;
use crate::heap::ObjRef;
use crate::threads::ThreadState;
use crate::value::Value;

pub(crate) const HANDLER_THREAD: &str = "Landroid/os/HandlerThread;";

/// Field on the HandlerThread instance that holds its Looper (`null` after quit).
const LOOPER_FIELD: &str = "rd_looper";

/// Instance dispatch for `HandlerThread` (and DEX subclasses). `Ok(None)` lets
/// the generic `Thread` path run (e.g. `start()` after the Looper is created).
pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    mref_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    match (name, sig) {
        ("<init>", _) if mref_class == HANDLER_THREAD => {
            // HandlerThread(String[, int priority])
            for a in args {
                if let Ok(Some(r)) = a.as_ref() {
                    if vm.heap.class_of(r)? == "Ljava/lang/String;" {
                        let n = vm.heap.as_str(r).ok().map(str::to_string);
                        if let Some(n) = n {
                            vm.threads.entry(recv).name = n;
                        }
                        break;
                    }
                }
            }
            Ok(Some(Value::Null))
        }
        ("start", "()V") => {
            if vm.threads.entry(recv).state == ThreadState::New {
                ensure_looper(vm, recv)?;
            }
            Ok(None)
        }
        ("getLooper", "()Landroid/os/Looper;") => {
            Ok(Some(match vm.heap.get_field(recv, LOOPER_FIELD) {
                Ok(Value::Obj(l)) => Value::Obj(l),
                _ => Value::Null,
            }))
        }
        ("quit", "()Z") | ("quitSafely", "()Z") => {
            let looper = match vm.heap.get_field(recv, LOOPER_FIELD) {
                Ok(Value::Obj(l)) => l,
                _ => return Ok(Some(Value::Int(0))),
            };
            let ok = quit_looper(vm, looper, name == "quitSafely");
            crate::err::put_field_result(vm.heap.put_field(recv, LOOPER_FIELD, Value::Null))?;
            Ok(Some(Value::Int(ok as i32)))
        }
        ("getThreadId", "()I") => {
            let id = vm.threads.entry(recv).id;
            Ok(Some(Value::Int(id as i32)))
        }
        _ => Ok(None),
    }
}

/// Creates the Looper of a started HandlerThread (once).
fn ensure_looper(vm: &mut Engine, thread: ObjRef) -> Result<ObjRef, VmExit> {
    if let Ok(Value::Obj(l)) = vm.heap.get_field(thread, LOOPER_FIELD) {
        return Ok(l);
    }
    let l = vm.alloc_looper()?;
    vm.fw.handler_loopers.push(l);
    vm.fw.looper_threads.push((l, thread));
    crate::err::put_field_result(vm.heap.put_field(thread, LOOPER_FIELD, Value::Obj(l)))?;
    Ok(l)
}

/// Quits a HandlerThread Looper. Returns `false` when it was not active (never
/// started, already quit, or the main Looper, which is never in these lists).
pub(crate) fn quit_looper(vm: &mut Engine, looper: ObjRef, safe: bool) -> bool {
    let active = vm.fw.handler_loopers.iter().position(|l| *l == looper);
    let quitting = vm.fw.quitting_loopers.contains(&looper);
    if active.is_none() && !quitting {
        return false;
    }
    if let Some(i) = active {
        vm.fw.handler_loopers.remove(i);
    }
    let now = vm.fw.clock_ms;
    let mut has_left = false;
    if let Some(HostObj::Looper { queue }) = vm.fw.objects.get_mut(&looper) {
        if safe {
            queue.retain(|m| m.when_ms <= now);
        } else {
            queue.clear();
        }
        has_left = !queue.is_empty();
    }
    if safe && has_left {
        if !quitting {
            vm.fw.quitting_loopers.push(looper);
        }
    } else {
        vm.fw.quitting_loopers.retain(|l| *l != looper);
    }
    true
}
