//! The `java.lang.Throwable` instance surface for host and DEX throwables.
//!
//! Throwable state lives in heap fields:
//! - `message`: the existing convention;
//! - `cause`, plus `rd_cause_set` to model AOSP's "cause == this means
//!   uninitialised" rule;
//! - `rd_suppressed`: a host ArrayList;
//! - `rd_stack`: a `StackTraceElement[]`.
//!
//! AOSP semantics implemented:
//! - `initCause` throws `IllegalStateException("Can't overwrite cause …")`
//!   if the cause was already set (including by a constructor), and
//!   `IllegalArgumentException("Self-causation not permitted")` when the
//!   cause is `this`.
//! - `addSuppressed(null)` throws NPE, and `addSuppressed(this)` throws IAE.
//! - `Throwable(Throwable cause)` sets `message = cause.toString()`.
//! - `getStackTrace` returns the last `setStackTrace` value, else an empty
//!   array. (The interpreter does not capture guest frames into throwables:
//!   a documented deviation.)
//! - `printStackTrace` writes `toString()` plus the `Caused by:` chain to the
//!   log as `W/System.err`.

use std::fmt::Write as _;

use crate::engine::Engine;
use crate::err::{Throwable, VmExit};
use crate::heap::{ElemKind, ObjRef};
use crate::value::Value;

fn field(vm: &Engine, r: ObjRef, f: &str) -> Value {
    vm.heap.get_field(r, f).unwrap_or(Value::Null)
}

fn put(vm: &mut Engine, r: ObjRef, f: &str, v: Value) -> Result<(), VmExit> {
    crate::err::put_field_result(vm.heap.put_field(r, f, v))
}

fn set_cause(vm: &mut Engine, recv: ObjRef, cause: Value) -> Result<(), VmExit> {
    put(vm, recv, "cause", cause)?;
    put(vm, recv, "rd_cause_set", Value::Int(1))
}

pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    recv_class: &str,
    name: &str,
    sig: &str,
    recv: ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if !matches!(
        name,
        "<init>"
            | "getCause"
            | "initCause"
            | "addSuppressed"
            | "getSuppressed"
            | "setStackTrace"
            | "getStackTrace"
            | "fillInStackTrace"
            | "printStackTrace"
    ) || !crate::intrinsics::is_throwable_class(vm, recv_class)
    {
        return Ok(None);
    }
    // Opt-in observability (`exception_trace`, i.e. --trace): a Throwable
    // constructed by guest code (Kotlin `!!`, checkNotNull, `throw new ..`)
    // records the guest call stack at construction, innermost first, in
    // `rd_trace`. Uncaught-exception logging prints it.
    if name == "<init>" && vm.config.exception_trace {
        let mut trace = String::new();
        for &(d, mi) in vm.frame_labels.iter().rev().take(16) {
            let _ = write!(&mut trace, "\n  at {}", vm.label_of(d, mi));
        }
        let t = crate::intrinsics::alloc_string(vm, trace)?;
        put(vm, recv, "rd_trace", Value::Obj(t))?;
    }
    match (name, sig) {
        ("<init>", "(Ljava/lang/String;Ljava/lang/Throwable;)V")
        | ("<init>", "(Ljava/lang/String;Ljava/lang/Throwable;ZZ)V") => {
            let msg = args.first().cloned().unwrap_or(Value::Null);
            put(vm, recv, "message", msg)?;
            set_cause(vm, recv, args.get(1).cloned().unwrap_or(Value::Null))?;
            Ok(Some(Value::Null))
        }
        // AssertionError(Object): mensagem = String.valueOf(detail) (null vira
        // "null"); se o detalhe for Throwable, ele também vira a causa.
        ("<init>", "(Ljava/lang/Object;)V") if recv_class == "Ljava/lang/AssertionError;" => {
            let detail = args.first().cloned().unwrap_or(Value::Null);
            let text = match detail {
                Value::Obj(d) => crate::intrinsics::object_to_string(vm, d)?,
                _ => "null".to_string(),
            };
            let msg = Value::Obj(crate::intrinsics::alloc_string(vm, text)?);
            put(vm, recv, "message", msg)?;
            if let Value::Obj(d) = detail {
                let is_throwable = vm
                    .heap
                    .class_of(d)
                    .ok()
                    .is_some_and(|c| crate::intrinsics::is_throwable_class(vm, c));
                if is_throwable {
                    set_cause(vm, recv, detail)?;
                }
            }
            Ok(Some(Value::Null))
        }
        ("<init>", "(Ljava/lang/Throwable;)V") => {
            let cause = args.first().cloned().unwrap_or(Value::Null);
            let msg = match cause {
                Value::Obj(c) => {
                    let s = crate::intrinsics::object_to_string(vm, c)?;
                    Value::Obj(crate::intrinsics::alloc_string(vm, s)?)
                }
                _ => Value::Null,
            };
            put(vm, recv, "message", msg)?;
            set_cause(vm, recv, cause)?;
            Ok(Some(Value::Null))
        }
        ("getCause", "()Ljava/lang/Throwable;") => {
            let c = field(vm, recv, "cause");
            Ok(Some(if c == Value::Obj(recv) {
                Value::Null
            } else {
                c
            }))
        }
        ("initCause", "(Ljava/lang/Throwable;)Ljava/lang/Throwable;") => {
            if field(vm, recv, "rd_cause_set") == Value::Int(1) {
                let cur = match field(vm, recv, "cause") {
                    Value::Obj(c) => crate::intrinsics::object_to_string(vm, c)?,
                    _ => "a null".into(),
                };
                let me = crate::intrinsics::object_to_string(vm, recv)?;
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IllegalStateException;",
                    format!("Can't overwrite cause with {cur}: {me}"),
                )));
            }
            let c = args.first().cloned().unwrap_or(Value::Null);
            if c == Value::Obj(recv) {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IllegalArgumentException;",
                    "Self-causation not permitted".to_string(),
                )));
            }
            set_cause(vm, recv, c)?;
            Ok(Some(Value::Obj(recv)))
        }
        ("addSuppressed", "(Ljava/lang/Throwable;)V") => {
            let t = match args.first() {
                Some(Value::Obj(t)) => *t,
                _ => {
                    return Err(VmExit::Exception(Throwable::new(
                        "Ljava/lang/NullPointerException;",
                        "Cannot suppress a null exception.".to_string(),
                    )))
                }
            };
            if t == recv {
                return Err(VmExit::Exception(Throwable::new(
                    "Ljava/lang/IllegalArgumentException;",
                    "Self-suppression not permitted".to_string(),
                )));
            }
            let mut list: Vec<Value> = match field(vm, recv, "rd_suppressed") {
                Value::Obj(a) => vm.heap.as_array_elems(a)?.0.to_vec(),
                _ => Vec::new(),
            };
            list.push(Value::Obj(t));
            let arr = new_array(vm, "Ljava/lang/Throwable;", list)?;
            put(vm, recv, "rd_suppressed", Value::Obj(arr))?;
            Ok(Some(Value::Null))
        }
        ("getSuppressed", "()[Ljava/lang/Throwable;") => {
            let list: Vec<Value> = match field(vm, recv, "rd_suppressed") {
                Value::Obj(a) => vm.heap.as_array_elems(a)?.0.to_vec(),
                _ => Vec::new(),
            };
            Ok(Some(Value::Obj(new_array(
                vm,
                "Ljava/lang/Throwable;",
                list,
            )?)))
        }
        ("setStackTrace", "([Ljava/lang/StackTraceElement;)V") => {
            match args.first() {
                Some(Value::Obj(a)) => {
                    let elems = vm.heap.as_array_elems(*a)?.0.to_vec();
                    let copy = new_array(vm, "Ljava/lang/StackTraceElement;", elems)?;
                    put(vm, recv, "rd_stack", Value::Obj(copy))?;
                }
                _ => {
                    return Err(VmExit::Exception(Throwable::new(
                        "Ljava/lang/NullPointerException;",
                        "stackTrace".to_string(),
                    )))
                }
            }
            Ok(Some(Value::Null))
        }
        ("getStackTrace", "()[Ljava/lang/StackTraceElement;") => {
            let elems: Vec<Value> = match field(vm, recv, "rd_stack") {
                Value::Obj(a) => vm.heap.as_array_elems(a)?.0.to_vec(),
                _ => Vec::new(),
            };
            Ok(Some(Value::Obj(new_array(
                vm,
                "Ljava/lang/StackTraceElement;",
                elems,
            )?)))
        }
        ("fillInStackTrace", "()Ljava/lang/Throwable;") => Ok(Some(Value::Obj(recv))),
        ("printStackTrace", _) => {
            let mut out = crate::intrinsics::object_to_string(vm, recv)?;
            let mut cur = recv;
            let mut seen = vec![recv];
            while let Value::Obj(c) = field(vm, cur, "cause") {
                if seen.contains(&c) {
                    break;
                }
                seen.push(c);
                out.push_str("\nCaused by: ");
                out.push_str(&crate::intrinsics::object_to_string(vm, c)?);
                cur = c;
            }
            for l in out.lines() {
                crate::intrinsics::host_print(vm, 1, &format!("W/System.err: {l}\n"));
            }
            Ok(Some(Value::Null))
        }
        _ => Ok(None),
    }
}

/// Renders an escaping throwable the way `printStackTrace` does: the head
/// line plus one `Caused by:` line per link of the cause chain.
///
/// The uncaught-exception contract used to report only `class: message`, so
/// every app that wraps a failure (`throw new InflateException(text, e)`,
/// `new IOException(msg, cause)`, Kotlin's `IllegalStateException(msg, e)`)
/// hid the real cause from the agent and from the CLI. The chain is walked
/// from the heap object (`cause` field), never from an invented list, and a
/// cycle or a missing object ends the walk.
pub(crate) fn describe_with_causes(vm: &Engine, t: &Throwable) -> String {
    let trace_of = |vm: &Engine, r: ObjRef| -> String {
        match field(vm, r, "rd_trace") {
            Value::Obj(t) => vm.heap.as_str(t).unwrap_or("").to_string(),
            _ => String::new(),
        }
    };
    let mut out = format!(
        "{}: {}",
        t.name_without_l(),
        t.message.clone().unwrap_or_default()
    );
    let Some(obj) = t.obj else {
        return out;
    };
    out.push_str(&trace_of(vm, obj));
    let mut cur = obj;
    let mut seen = vec![obj];
    for _ in 0..8 {
        let Value::Obj(c) = field(vm, cur, "cause") else {
            break;
        };
        if seen.contains(&c) {
            break;
        }
        seen.push(c);
        let class = match vm.heap.class_of(c) {
            Ok(k) => k
                .trim_start_matches('L')
                .trim_end_matches(';')
                .replace('/', "."),
            Err(_) => "?".to_string(),
        };
        let msg = match field(vm, c, "message") {
            Value::Obj(m) => vm.heap.as_str(m).unwrap_or("").to_string(),
            _ => String::new(),
        };
        if msg.is_empty() {
            out.push_str(&format!("\nCaused by: {class}"));
        } else {
            out.push_str(&format!("\nCaused by: {class}: {msg}"));
        }
        out.push_str(&trace_of(vm, c));
        cur = c;
    }
    out
}

fn new_array(vm: &mut Engine, elem: &str, vals: Vec<Value>) -> Result<ObjRef, VmExit> {
    // alloc_array leaves the element vector empty: assign it whole (indexing
    // panicked and killed the MCP server on getStackTrace/getSuppressed)
    let a = vm.heap.alloc_array(ElemKind::Obj, vals.len(), elem)?;
    *vm.heap.array_elems_mut(a)? = vals;
    Ok(a)
}
