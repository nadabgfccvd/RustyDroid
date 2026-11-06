//! `java.util.logging.Logger` (the string-level subset).
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner): the Guava
//! `AbstractFuture` bundled in the APK (used by CameraX's `ListenableFuture`)
//! calls `Logger.getLogger(getName())` in its static initializer. The call was
//! `NOT_FOUND`, which surfaced as `ExceptionInInitializerError`.
//!
//! Model, from AOSP (`core/java/com/android/internal/logging/AndroidHandler.java`):
//! - `getLogger(name)` returns one logger per name (the LogManager registry).
//! - Records reach `android.util.Log` through `Log.println(priority, tag, msg)`:
//!   - the tag is `loggerNameToTag(name)`, at most 23 chars (the last segment if it
//!     fits, else the last 23 chars);
//!   - priority: SEVERE (>= 1000) -> ERROR, WARNING (>= 900) -> WARN, INFO (>= 800)
//!     -> INFO, anything lower -> DEBUG.
//! - The root logger level is INFO, so `config`, `fine`, `finer` and `finest`
//!   records are dropped before they reach a handler.
//!
//! Regression (2026-11-06, Comprix v1.6.0, tab Scanner, second round): CameraX
//! calls `Logger.log(Level.SEVERE, msg, Throwable)` and `logp(Level, ...)`, so
//! `java.util.logging.Level` is modelled too: the constants `Level.SEVERE`,
//! `WARNING`, `INFO`, `CONFIG`, `FINE`, `FINER`, `FINEST` and `intValue()`.
//! Records from `log`/`logp` take the same root-level filter and priority as
//! the `warning`/`severe`/... shorthands.
//!
//! Not modelled: `setLevel`, `isLoggable`, handlers, formatters, and the
//! stack trace of the `Throwable` argument (only the message reaches
//! `Log.println`). Those calls stay a typed `NOT_FOUND` instead of being guessed.

use crate::engine::Engine;
use crate::err::VmExit;
use crate::value::Value;

pub(crate) const LOGGER: &str = "Ljava/util/logging/Logger;";
pub(crate) const LEVEL: &str = "Ljava/util/logging/Level;";

const FIELD_NAME: &str = "rd_name";
const LEVEL_FIELD: &str = "rd_value";
const ROOT_LEVEL: i32 = 800; // Level.INFO: the default root level

/// `AndroidHandler.loggerNameToTag`.
pub(crate) fn logger_name_to_tag(name: &str) -> String {
    if name.len() <= 23 {
        return name.to_string();
    }
    let last_period = name.rfind('.');
    match last_period {
        Some(p) if name.len() - (p + 1) <= 23 => name[p + 1..].to_string(),
        _ => name[name.len() - 23..].to_string(),
    }
}

/// `AndroidHandler.getAndroidLevel`.
pub(crate) fn android_priority(level: i32) -> i32 {
    if level >= 1000 {
        6 // Log.ERROR
    } else if level >= 900 {
        5 // Log.WARN
    } else if level >= 800 {
        4 // Log.INFO
    } else {
        3 // Log.DEBUG
    }
}

/// `Logger.getLogger(String)`: one object per name.
pub(crate) fn static_intrinsic(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name != "getLogger" || sig != "(Ljava/lang/String;)Ljava/util/logging/Logger;" {
        return Ok(None);
    }
    let Some(Value::Obj(n)) = args.first().cloned() else {
        return Ok(Some(Value::Null));
    };
    let key_name = vm.heap.as_str(n).ok().unwrap_or_default().to_string();
    let key = (LOGGER.to_string(), format!("rd_logger:{key_name}"));
    if let Some(Value::Obj(r)) = vm.statics.get(&key) {
        return Ok(Some(Value::Obj(*r)));
    }
    let r = vm
        .heap
        .alloc_instance(LOGGER.to_string(), Vec::new())
        .map_err(crate::err::oom_throwable)?;
    crate::err::put_field_result(vm.heap.put_field(r, FIELD_NAME, Value::Obj(n)))?;
    vm.statics.insert(key, Value::Obj(r));
    Ok(Some(Value::Obj(r)))
}

/// `java.util.logging.Level` constants. Each level is one object, with its
/// int value in `rd_value`.
fn level_value_of(field: &str) -> Option<i32> {
    Some(match field {
        "SEVERE" => 1000,
        "WARNING" => 900,
        "INFO" => 800,
        "CONFIG" => 700,
        "FINE" => 500,
        "FINER" => 400,
        "FINEST" => 300,
        _ => return None,
    })
}

fn level_obj(vm: &mut Engine, value: i32) -> Result<crate::heap::ObjRef, VmExit> {
    let key = (LEVEL.to_string(), format!("rd_level:{value}"));
    if let Some(Value::Obj(r)) = vm.statics.get(&key) {
        return Ok(*r);
    }
    let r = vm
        .heap
        .alloc_instance(
            LEVEL.to_string(),
            vec![(LEVEL_FIELD.to_string(), Value::Int(value))],
        )
        .map_err(crate::err::oom_throwable)?;
    vm.statics.insert(key, Value::Obj(r));
    Ok(r)
}

/// `Level.SEVERE` and the other standard level constants (`sget`).
pub(crate) fn host_static(
    vm: &mut Engine,
    owner: &str,
    field: &str,
) -> Result<Option<Value>, VmExit> {
    if owner != LEVEL {
        return Ok(None);
    }
    let Some(value) = level_value_of(field) else {
        return Ok(None);
    };
    Ok(Some(Value::Obj(level_obj(vm, value)?)))
}

/// Integer value of a `Level` object; `None` for `null` or a foreign object.
fn level_int(vm: &mut Engine, level: &Value) -> Option<i32> {
    let Value::Obj(r) = level else {
        return None;
    };
    match vm.heap.get_field(*r, LEVEL_FIELD) {
        Ok(Value::Int(v)) => Some(v),
        _ => None,
    }
}

/// Instance methods of `java.util.logging.Level`.
pub(crate) fn level_instance_intrinsic(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: crate::heap::ObjRef,
) -> Result<Option<Value>, VmExit> {
    if name == "intValue" && sig == "()I" {
        return Ok(Some(
            vm.heap
                .get_field(recv, LEVEL_FIELD)
                .unwrap_or(Value::Int(0)),
        ));
    }
    Ok(None)
}

/// Instance methods of `java.util.logging.Logger`.
pub(crate) fn instance_intrinsic(
    vm: &mut Engine,
    name: &str,
    sig: &str,
    recv: crate::heap::ObjRef,
    args: &[Value],
) -> Result<Option<Value>, VmExit> {
    if name == "getName" && sig == "()Ljava/lang/String;" {
        return Ok(Some(
            vm.heap.get_field(recv, FIELD_NAME).unwrap_or(Value::Null),
        ));
    }
    // (level, message): the shorthands carry the level in the method name,
    // `log`/`logp` take it as the first argument.
    let (level, msg, thrown) = match (name, sig) {
        ("severe", "(Ljava/lang/String;)V") => (1000, args.first(), None),
        ("warning", "(Ljava/lang/String;)V") => (900, args.first(), None),
        ("info", "(Ljava/lang/String;)V") => (800, args.first(), None),
        ("config", "(Ljava/lang/String;)V") => (700, args.first(), None),
        ("fine", "(Ljava/lang/String;)V") => (500, args.first(), None),
        ("finer", "(Ljava/lang/String;)V") => (400, args.first(), None),
        ("finest", "(Ljava/lang/String;)V") => (300, args.first(), None),
        ("log", "(Ljava/util/logging/Level;Ljava/lang/String;)V")
        | ("log", "(Ljava/util/logging/Level;Ljava/lang/String;Ljava/lang/Throwable;)V") => {
            let Some(level) = args.first().and_then(|l| level_int(vm, l)) else {
                return Ok(Some(Value::Null));
            };
            (level, args.get(1), args.get(2))
        }
        (
            "logp",
            "(Ljava/util/logging/Level;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
        )
        | (
            "logp",
            "(Ljava/util/logging/Level;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/Throwable;)V",
        ) => {
            let Some(level) = args.first().and_then(|l| level_int(vm, l)) else {
                return Ok(Some(Value::Null));
            };
            (level, args.get(3), args.get(4))
        }
        _ => return Ok(None),
    };
    if level < ROOT_LEVEL {
        return Ok(Some(Value::Null)); // dropped by the root level
    }
    let logger_name = match vm.heap.get_field(recv, FIELD_NAME) {
        Ok(Value::Obj(n)) => vm.heap.as_str(n).ok().unwrap_or_default().to_string(),
        _ => "null".to_string(),
    };
    let tag = logger_name_to_tag(&logger_name);
    let tag_v = vm
        .heap
        .alloc_string(tag)
        .map(Value::Obj)
        .map_err(crate::err::oom_throwable)?;
    let msg = log_message(vm, msg.cloned().unwrap_or(Value::Null), thrown.cloned());
    vm.invoke_static(
        "Landroid/util/Log;",
        "println",
        "(ILjava/lang/String;Ljava/lang/String;)I",
        &[Value::Int(android_priority(level)), tag_v, msg],
    )?;
    Ok(Some(Value::Null))
}

/// Message as `AndroidHandler` prints it: the record text, then the
/// `Throwable` (the stack trace there; `toString` here, since frames are not
/// kept). A `toString` that throws is skipped, so logging never fails.
fn log_message(vm: &mut Engine, msg: Value, thrown: Option<Value>) -> Value {
    let Some(Value::Obj(t)) = thrown else {
        return msg;
    };
    let Ok(Value::Obj(ts)) = vm.invoke_instance(t, "toString", "()Ljava/lang/String;", &[]) else {
        return msg;
    };
    let thrown_text = vm.heap.as_str(ts).ok().unwrap_or_default().to_string();
    let text = match msg {
        Value::Obj(m) => format!(
            "{}\n{}",
            vm.heap.as_str(m).ok().unwrap_or_default(),
            thrown_text
        ),
        _ => format!("null\n{thrown_text}"),
    };
    vm.heap
        .alloc_string(text)
        .map(Value::Obj)
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_names_are_the_tag() {
        assert_eq!(logger_name_to_tag("a1.h"), "a1.h");
    }

    #[test]
    fn long_names_use_the_last_segment_when_it_fits() {
        assert_eq!(
            logger_name_to_tag("com.example.camera.internal.ListenableFutureImpl"),
            "ListenableFutureImpl"
        );
    }

    #[test]
    fn long_segments_use_the_last_23_chars() {
        let n = "x.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let t = logger_name_to_tag(n);
        assert_eq!(t.len(), 23);
        assert!(n.ends_with(&t));
    }

    #[test]
    fn standard_level_constants_have_the_jdk_values() {
        assert_eq!(level_value_of("SEVERE"), Some(1000));
        assert_eq!(level_value_of("WARNING"), Some(900));
        assert_eq!(level_value_of("INFO"), Some(800));
        assert_eq!(level_value_of("CONFIG"), Some(700));
        assert_eq!(level_value_of("FINE"), Some(500));
        assert_eq!(level_value_of("FINER"), Some(400));
        assert_eq!(level_value_of("FINEST"), Some(300));
        // Not a standard constant: not invented.
        assert_eq!(level_value_of("ALL"), None);
        assert_eq!(level_value_of("OFF"), None);
    }

    #[test]
    fn log_with_level_maps_to_the_same_android_priority() {
        // Guava's `AbstractFuture` logs `Level.SEVERE`, CameraX logs `WARNING`.
        assert_eq!(android_priority(level_value_of("SEVERE").unwrap()), 6);
        assert_eq!(android_priority(level_value_of("WARNING").unwrap()), 5);
        // Below the root level (INFO): dropped before reaching Log.
        assert!(level_value_of("FINE").unwrap() < ROOT_LEVEL);
    }

    #[test]
    fn priorities_follow_the_aosp_thresholds() {
        assert_eq!(android_priority(1000), 6);
        assert_eq!(android_priority(900), 5);
        assert_eq!(android_priority(800), 4);
        assert_eq!(android_priority(700), 3);
    }
}
