//! Regression coverage for Java's `Throwable.toString()` contract on host
//! exceptions and the preservation of app-defined overrides.

use rd_testsupport::vm::*;
use rd_vm::{Engine, Value, VmConfig};

fn throwable(e: &mut Engine, class: &str, message: Option<&str>) -> u32 {
    let obj = e
        .heap
        .alloc_instance(class.to_string(), Vec::new())
        .expect("allocate throwable");
    if let Some(message) = message {
        let message = Value::Obj(e.heap.alloc_string(message.to_string()).expect("message"));
        e.heap
            .put_field(obj, "message", message)
            .expect("set message");
    }
    obj
}

fn string_method(e: &mut Engine, obj: u32, method: &str) -> String {
    let got = e
        .invoke_instance(obj, method, "()Ljava/lang/String;", &[])
        .expect("string-returning Throwable method");
    let Value::Obj(text) = got else {
        panic!("{method} must return a String, got {got:?}");
    };
    e.heap.as_str(text).expect("string value").to_string()
}

fn to_string(e: &mut Engine, obj: u32) -> String {
    string_method(e, obj, "toString")
}

#[test]
fn host_throwable_to_string_uses_class_and_optional_message() {
    let mut e = Engine::new(Vec::new(), VmConfig::default());
    let without_message = throwable(&mut e, "Ljava/lang/IllegalStateException;", None);
    assert_eq!(
        to_string(&mut e, without_message),
        "java.lang.IllegalStateException"
    );

    let with_message = throwable(
        &mut e,
        "Ljava/lang/IllegalArgumentException;",
        Some("invalid argument"),
    );
    assert_eq!(
        to_string(&mut e, with_message),
        "java.lang.IllegalArgumentException: invalid argument"
    );
}

#[test]
fn dex_throwable_override_still_dispatches_to_app_code() {
    let mut b = DexBuilder::new();
    let cls = b.class("LCustomException;", "Ljava/lang/IllegalStateException;");
    let text = b.intern("custom exception text") as u16;
    let mut code = op21c(0x1a, 0, text); // const-string v0
    code.extend(op11x(0x11, 0)); // return-object v0
    let blob = b.code(2, 1, 1, code);
    b.r#virtual(
        cls,
        "toString",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC,
        Some(blob),
    );

    let mut e = engine_of(&b);
    e.ensure_initialized("LCustomException;")
        .expect("initialize custom exception");
    let obj = e
        .new_dex_instance_pub("LCustomException;")
        .expect("allocate custom exception");
    assert_eq!(to_string(&mut e, obj), "custom exception text");
}

#[test]
fn inherited_localized_message_dispatches_to_dex_get_message() {
    let mut b = DexBuilder::new();
    let cls = b.class("LMessageException;", "Ljava/lang/IllegalStateException;");
    let text = b.intern("overridden message") as u16;
    let mut code = op21c(0x1a, 0, text); // const-string v0
    code.extend(op11x(0x11, 0)); // return-object v0
    let blob = b.code(2, 1, 1, code);
    b.r#virtual(
        cls,
        "getMessage",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC,
        Some(blob),
    );

    let mut e = engine_of(&b);
    e.ensure_initialized("LMessageException;")
        .expect("initialize message exception");
    let obj = e
        .new_dex_instance_pub("LMessageException;")
        .expect("allocate message exception");
    let base = Value::Obj(e.heap.alloc_string("base text".to_string()).expect("base"));
    e.heap.put_field(obj, "message", base).expect("set base");
    assert_eq!(
        string_method(&mut e, obj, "getLocalizedMessage"),
        "overridden message"
    );
    assert_eq!(
        to_string(&mut e, obj),
        "MessageException: overridden message"
    );
}

#[test]
fn throwable_to_string_uses_dex_localized_message_override() {
    let mut b = DexBuilder::new();
    let cls = b.class("LLocalizedException;", "Ljava/lang/IllegalStateException;");
    let text = b.intern("localized text") as u16;
    let mut code = op21c(0x1a, 0, text); // const-string v0
    code.extend(op11x(0x11, 0)); // return-object v0
    let blob = b.code(2, 1, 1, code);
    b.r#virtual(
        cls,
        "getLocalizedMessage",
        "Ljava/lang/String;",
        vec![],
        ACC_PUBLIC,
        Some(blob),
    );

    let mut e = engine_of(&b);
    e.ensure_initialized("LLocalizedException;")
        .expect("initialize localized exception");
    let obj = e
        .new_dex_instance_pub("LLocalizedException;")
        .expect("allocate localized exception");
    let base = Value::Obj(e.heap.alloc_string("base text".to_string()).expect("base"));
    e.heap.put_field(obj, "message", base).expect("set base");
    assert_eq!(to_string(&mut e, obj), "LocalizedException: localized text");
}
