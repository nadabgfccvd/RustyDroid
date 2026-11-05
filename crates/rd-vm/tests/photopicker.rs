//! Apêndice D (S10) — photo picker com resultado programável
//! (`photopicker.rs`). O lançamento sai de bytecode DEX
//! (`invoke-virtual Activity.startActivityForResult`), e o resultado volta
//! ao `onActivityResult` de uma subclasse de Activity que grava rq, rc e data
//! em estáticos. Cobre fila, cancelamento, FIFO, ações de picker, ações fora
//! do picker e o `IntentMeta` que `getData()` lê.

mod common;

use common::{engine_of, op10x, op21c, op35c, DexBuilder, SVal, ACC_PUBLIC};
use rd_vm::photopicker::{self as pp, PickerAnswer};
use rd_vm::value::Value;
use rd_vm::{Engine, ObjRef};

const ACTIVITY: &str = "Landroid/app/Activity;";
const INTENT: &str = "Landroid/content/Intent;";
const PICK_ACT: &str = "LPickAct;";
const LAUNCH_SIG: &str = "(Landroid/content/Intent;I)V";
const START_FOR_RESULT_PROTO: (&str, &[&str]) = ("V", &["Landroid/content/Intent;", "I"]);

/// `LPickAct;` (subclasse de Activity):
/// - `launch(Intent, int)`: `invoke-virtual` de `startActivityForResult`.
/// - `onActivityResult(int, int, Intent)`: grava rq, rc e data em estáticos.
fn pick_activity_dex() -> DexBuilder {
    let mut b = DexBuilder::new();
    let cls = b.class(PICK_ACT, ACTIVITY);
    b.static_field(cls, "rq", "I", SVal::Int(0));
    b.static_field(cls, "rc", "I", SVal::Int(0));
    b.static_field(cls, "data", INTENT, SVal::Int(0));
    let f_rq = b.field_idx(PICK_ACT, "I", "rq");
    let f_rc = b.field_idx(PICK_ACT, "I", "rc");
    let f_data = b.field_idx(PICK_ACT, INTENT, "data");

    // v0 = this, v1 = requestCode, v2 = resultCode, v3 = data
    let mut on_result = op21c(0x67, 1, f_rq); // sput v1, rq
    on_result.extend(op21c(0x67, 2, f_rc)); // sput v2, rc
    on_result.extend(op21c(0x69, 3, f_data)); // sput-object v3, data
    on_result.extend(op10x(0x0E)); // return-void
    b.r#virtual(
        cls,
        "onActivityResult",
        "V",
        vec!["I", "I", INTENT],
        ACC_PUBLIC,
        Some(b.code(4, 4, 0, on_result)),
    );

    // v0 = this, v1 = intent, v2 = requestCode
    let proto = b.proto_idx(
        START_FOR_RESULT_PROTO.0,
        START_FOR_RESULT_PROTO
            .1
            .iter()
            .map(|s| s.to_string())
            .collect(),
    );
    let start = b.method_idx(ACTIVITY, proto, "startActivityForResult");
    let mut launch = op35c(0x6e, 3, start, [0, 1, 2, 0, 0]); // invoke-virtual {v0,v1,v2}
    launch.extend(op10x(0x0E));
    b.r#virtual(
        cls,
        "launch",
        "V",
        vec![INTENT, "I"],
        ACC_PUBLIC,
        Some(b.code(3, 3, 3, launch)),
    );
    b
}

fn setup() -> Engine {
    engine_of(&pick_activity_dex())
}

fn new_activity(e: &mut Engine) -> ObjRef {
    let act = e
        .heap
        .alloc_instance(PICK_ACT.to_string(), Vec::new())
        .expect("alloc activity");
    e.attach_host_state(ACTIVITY, act).expect("attach activity");
    act
}

/// `new Intent(action)` pelo caminho de dispatch real.
fn intent_with_action(e: &mut Engine, action: &str) -> ObjRef {
    let intent = e
        .heap
        .alloc_instance(INTENT.to_string(), Vec::new())
        .expect("alloc intent");
    let s = e
        .heap
        .alloc_string(action.to_string())
        .expect("alloc string");
    e.invoke_instance(intent, "<init>", "(Ljava/lang/String;)V", &[Value::Obj(s)])
        .expect("Intent(String)");
    intent
}

fn launch(
    e: &mut Engine,
    act: ObjRef,
    intent: ObjRef,
    request: i32,
) -> Result<Value, rd_vm::err::VmExit> {
    e.invoke_instance(
        act,
        "launch",
        LAUNCH_SIG,
        &[Value::Obj(intent), Value::Int(request)],
    )
}

fn static_int(e: &Engine, field: &str) -> i32 {
    match e.statics.get(&(PICK_ACT.to_string(), field.to_string())) {
        Some(Value::Int(n)) => *n,
        other => panic!("estático {field}: {other:?}"),
    }
}

/// Intent entregue ao onActivityResult; `None` se nulo ou não definido.
fn static_data(e: &Engine) -> Option<ObjRef> {
    match e.statics.get(&(PICK_ACT.to_string(), "data".to_string())) {
        Some(Value::Obj(r)) => Some(*r),
        _ => None,
    }
}

fn data_text(e: &mut Engine, intent: ObjRef) -> Option<String> {
    let uri = match e
        .invoke_instance(intent, "getData", "()Landroid/net/Uri;", &[])
        .expect("getData")
    {
        Value::Obj(u) => u,
        _ => return None,
    };
    pp::uri_text(e, uri)
}

fn expect_not_implemented(r: Result<Value, rd_vm::err::VmExit>) {
    match r {
        Err(err) => assert!(
            format!("{err:?}").contains("NOT_IMPLEMENTED"),
            "esperava NOT_IMPLEMENTED, veio {err:?}"
        ),
        Ok(v) => panic!("esperava erro, veio {v:?}"),
    }
}

const URI_A: &str = "content://media/picker/0/com.android.providers.media.photopicker/media/42";
const URI_B: &str = "content://media/picker/0/com.android.providers.media.photopicker/media/43";

#[test]
fn picker_without_programmed_answer_cancels_with_null_data() {
    let mut e = setup();
    let act = new_activity(&mut e);
    let intent = intent_with_action(&mut e, pp::ACTION_PICK_IMAGES);
    launch(&mut e, act, intent, 7).expect("launch PICK_IMAGES");
    assert_eq!(static_int(&e, "rq"), 7, "requestCode entregue");
    assert_eq!(
        static_int(&e, "rc"),
        pp::RESULT_CANCELED,
        "sem resultado: cancelado"
    );
    assert!(static_data(&e).is_none(), "cancelado não traz data");
}

#[test]
fn programmed_uri_is_delivered_as_result_ok_with_getdata() {
    let mut e = setup();
    let act = new_activity(&mut e);
    pp::enqueue(&mut e, PickerAnswer::Uri(URI_A.to_string()));
    let intent = intent_with_action(&mut e, pp::ACTION_PICK_IMAGES);
    launch(&mut e, act, intent, 9).expect("launch");
    assert_eq!(static_int(&e, "rq"), 9);
    assert_eq!(
        static_int(&e, "rc"),
        pp::RESULT_OK,
        "URI programada: RESULT_OK"
    );
    let result = static_data(&e).expect("data entregue");
    assert_eq!(data_text(&mut e, result).as_deref(), Some(URI_A));
    assert_eq!(pp::pending(&e), 0, "resultado consumido");
}

#[test]
fn answers_are_fifo_and_empty_queue_cancels_again() {
    let mut e = setup();
    let act = new_activity(&mut e);
    pp::enqueue(&mut e, PickerAnswer::Uri(URI_A.to_string()));
    pp::enqueue(&mut e, PickerAnswer::Uri(URI_B.to_string()));
    let intent = intent_with_action(&mut e, pp::ACTION_PICK_IMAGES);

    launch(&mut e, act, intent, 1).expect("launch 1");
    let d1 = static_data(&e).expect("data 1");
    assert_eq!(
        data_text(&mut e, d1).as_deref(),
        Some(URI_A),
        "primeiro é A"
    );

    launch(&mut e, act, intent, 2).expect("launch 2");
    let d2 = static_data(&e).expect("data 2");
    assert_eq!(data_text(&mut e, d2).as_deref(), Some(URI_B), "depois B");

    launch(&mut e, act, intent, 3).expect("launch 3");
    assert_eq!(
        static_int(&e, "rc"),
        pp::RESULT_CANCELED,
        "fila vazia cancela"
    );
    assert!(static_data(&e).is_none());
}

#[test]
fn get_content_and_open_document_are_picker_actions() {
    let mut e = setup();
    let act = new_activity(&mut e);
    pp::enqueue(&mut e, PickerAnswer::Uri(URI_A.to_string()));
    let gc = intent_with_action(&mut e, pp::ACTION_GET_CONTENT);
    launch(&mut e, act, gc, 4).expect("GET_CONTENT");
    assert_eq!(static_int(&e, "rc"), pp::RESULT_OK);

    pp::enqueue(&mut e, PickerAnswer::Uri(URI_B.to_string()));
    let od = intent_with_action(&mut e, pp::ACTION_OPEN_DOCUMENT);
    launch(&mut e, act, od, 5).expect("OPEN_DOCUMENT");
    let d = static_data(&e).expect("data OPEN_DOCUMENT");
    assert_eq!(data_text(&mut e, d).as_deref(), Some(URI_B));
}

#[test]
fn non_picker_implicit_intent_stays_not_implemented_and_keeps_queue() {
    let mut e = setup();
    let act = new_activity(&mut e);
    pp::enqueue(&mut e, PickerAnswer::Uri(URI_A.to_string()));
    let view = intent_with_action(&mut e, "android.intent.action.VIEW");
    expect_not_implemented(launch(&mut e, act, view, 2));
    assert_eq!(
        pp::pending(&e),
        1,
        "ação fora do picker não consome resultado"
    );
    assert!(static_data(&e).is_none(), "nada entregue");
}

#[test]
fn intent_without_action_is_not_a_picker_launch() {
    let mut e = setup();
    let act = new_activity(&mut e);
    let bare = e
        .heap
        .alloc_instance(INTENT.to_string(), Vec::new())
        .expect("alloc intent");
    expect_not_implemented(launch(&mut e, act, bare, 1));
}

#[test]
fn result_intent_exposes_uri_via_getdata_and_null_when_cancelled() {
    let mut e = setup();
    let act = new_activity(&mut e);
    pp::enqueue(&mut e, PickerAnswer::Uri(URI_A.to_string()));
    let intent = intent_with_action(&mut e, pp::ACTION_PICK_IMAGES);
    launch(&mut e, act, intent, 1).expect("launch");
    let r = static_data(&e).expect("resultado");
    let ty = e
        .invoke_instance(r, "getType", "()Ljava/lang/String;", &[])
        .expect("getType");
    assert!(matches!(ty, Value::Null), "resultado não define tipo");
    assert_eq!(data_text(&mut e, r).as_deref(), Some(URI_A));

    // O intent original não recebe dados do resultado.
    assert!(matches!(
        e.invoke_instance(intent, "getData", "()Landroid/net/Uri;", &[]),
        Ok(Value::Null)
    ));
}
