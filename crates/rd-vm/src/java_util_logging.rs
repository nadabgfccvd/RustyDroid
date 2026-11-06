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
//! Not modelled: `Level` objects (`log(Level, ...)`, `setLevel`, `isLoggable`,
//! handlers, formatters). Those calls stay a typed `NOT_FOUND` instead of
//! being guessed.

use crate::engine::Engine;
use crate::err::VmExit;
use crate::value::Value;

pub(crate) const LOGGER: &str = "Ljava/util/logging/Logger;";

const FIELD_NAME: &str = "rd_name";
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
    let level = match (name, sig) {
        ("severe", "(Ljava/lang/String;)V") => 1000,
        ("warning", "(Ljava/lang/String;)V") => 900,
        ("info", "(Ljava/lang/String;)V") => 800,
        ("config", "(Ljava/lang/String;)V") => 700,
        ("fine", "(Ljava/lang/String;)V") => 500,
        ("finer", "(Ljava/lang/String;)V") => 400,
        ("finest", "(Ljava/lang/String;)V") => 300,
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
    let msg = args.first().cloned().unwrap_or(Value::Null);
    vm.invoke_static(
        "Landroid/util/Log;",
        "println",
        "(ILjava/lang/String;Ljava/lang/String;)I",
        &[Value::Int(android_priority(level)), tag_v, msg],
    )?;
    Ok(Some(Value::Null))
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
    fn priorities_follow_the_aosp_thresholds() {
        assert_eq!(android_priority(1000), 6);
        assert_eq!(android_priority(900), 5);
        assert_eq!(android_priority(800), 4);
        assert_eq!(android_priority(700), 3);
    }
}
