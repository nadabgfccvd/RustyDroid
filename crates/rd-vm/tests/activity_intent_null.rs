//! Regressão (RELATORIO F-Droid, commit 65bedb6): `Activity.getIntent()` sem
//! intent devolvia `NOT_IMPLEMENTED` ("getIntent antes do launch"). A AOSP
//! devolve null, e é isso que o AndroidX lê durante a composição inicial depois
//! de o IntentRouter limpar o intent com `setIntent(null)`.

use rd_testsupport::vm::{engine_of, DexBuilder};
use rd_vm::{Engine, ObjRef, Value};

const ACTIVITY: &str = "Landroid/app/Activity;";
const INTENT: &str = "Landroid/content/Intent;";
const GET_INTENT: &str = "()Landroid/content/Intent;";
const SET_INTENT: &str = "(Landroid/content/Intent;)V";

fn new_activity(e: &mut Engine) -> ObjRef {
    let act = e
        .heap
        .alloc_instance(ACTIVITY.to_string(), Vec::new())
        .expect("alloc activity");
    e.attach_host_state(ACTIVITY, act).expect("attach activity");
    act
}

#[test]
fn get_intent_without_an_intent_returns_null_like_aosp() {
    let mut e = engine_of(&DexBuilder::new());
    let act = new_activity(&mut e);
    let r = e.invoke_instance(act, "getIntent", GET_INTENT, &[]);
    assert!(matches!(r, Ok(Value::Null)), "esperava null, veio {r:?}");
}

#[test]
fn set_intent_then_get_returns_it_and_null_clears_it() {
    let mut e = engine_of(&DexBuilder::new());
    let act = new_activity(&mut e);
    let intent = e
        .heap
        .alloc_instance(INTENT.to_string(), Vec::new())
        .expect("alloc intent");

    e.invoke_instance(act, "setIntent", SET_INTENT, &[Value::Obj(intent)])
        .expect("setIntent(intent)");
    let r = e.invoke_instance(act, "getIntent", GET_INTENT, &[]);
    assert!(matches!(r, Ok(Value::Obj(got)) if got == intent), "{r:?}");

    // IntentRouter consumes the deep link and clears the intent.
    e.invoke_instance(act, "setIntent", SET_INTENT, &[Value::Null])
        .expect("setIntent(null)");
    let r = e.invoke_instance(act, "getIntent", GET_INTENT, &[]);
    assert!(
        matches!(r, Ok(Value::Null)),
        "esperava null após setIntent(null), veio {r:?}"
    );
}
