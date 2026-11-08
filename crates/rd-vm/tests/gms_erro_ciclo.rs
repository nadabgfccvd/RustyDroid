//! Apêndice E — critério de erro e ciclo de vida (`gms_erro_ciclo.rs`).
//!
//! Serviços com caminho de falha programável: a Task falha com exceção do
//! tipo da biblioteca, o estado de erro não vaza para o pedido seguinte e a
//! recuperação (limpar a falha) volta ao sucesso. Nenhuma rede: o estado é
//! programado pelo hospedeiro (`Engine::gms_*`) e lido pelo convidado.

#[macro_use]
mod common;

use common::*;
use rd_vm::gms::*;
use rd_vm::value::Value;
use rd_vm::{Engine, VmConfig};

fn engine() -> Engine {
    Engine::new(Vec::new(), VmConfig::default())
}
fn s(e: &mut Engine, txt: &str) -> Value {
    Value::Obj(e.heap.alloc_string(txt.to_string()).unwrap())
}
fn ov(v: Option<Value>) -> u32 {
    match v {
        Some(Value::Obj(r)) => r,
        other => panic!("esperava Obj, veio {other:?}"),
    }
}
fn iv(v: Option<Value>) -> i32 {
    match v {
        Some(Value::Int(n)) => n,
        other => panic!("esperava Int, veio {other:?}"),
    }
}
fn obj(e: &mut Engine, class: &str) -> u32 {
    e.heap
        .alloc_instance(class.to_string(), Vec::new())
        .unwrap()
}
fn ctx(e: &mut Engine) -> Value {
    Value::Obj(
        e.heap
            .alloc_instance("Landroid/content/Context;".into(), Vec::new())
            .unwrap(),
    )
}
fn sv(e: &Engine, v: Option<Value>) -> String {
    match v {
        Some(Value::Obj(r)) => e.heap.as_str(r).unwrap_or("").to_string(),
        other => panic!("esperava String, veio {other:?}"),
    }
}

macro_rules! ci {
    ($e:expr, $cls:expr, $name:expr, $sig:expr, $recv:expr, [$($a:expr),* $(,)?]) => {{
        let __args: Vec<Value> = vec![$($a),*];
        call_instance(&mut $e, $cls, $name, $sig, $recv, &__args)
    }};
}
macro_rules! cs {
    ($e:expr, $cls:expr, $name:expr, $sig:expr, [$($a:expr),* $(,)?]) => {{
        let __args: Vec<Value> = vec![$($a),*];
        call_static(&mut $e, $cls, $name, $sig, &__args)
    }};
}

/// Pede um token de integridade com o nonce dado e devolve a Task.
fn request_token(mut e: &mut Engine, manager: u32, nonce: &str) -> u32 {
    let builder = ov(cs!(
        e,
        INTEGRITY_TOKEN_REQUEST,
        "newBuilder",
        "()Lcom/google/android/play/core/integrity/IntegrityTokenRequest$Builder;",
        []
    )
    .unwrap());
    let n = s(e, nonce);
    ci!(
        e,
        INTEGRITY_TOKEN_BUILDER,
        "setNonce",
        "(Ljava/lang/String;)Lcom/google/android/play/core/integrity/IntegrityTokenRequest$Builder;",
        builder,
        [n]
    )
    .unwrap();
    let request = ov(ci!(
        e,
        INTEGRITY_TOKEN_BUILDER,
        "build",
        "()Lcom/google/android/play/core/integrity/IntegrityTokenRequest;",
        builder,
        []
    )
    .unwrap());
    ov(ci!(
        e,
        "Lcom/google/android/play/core/integrity/IntegrityManager;",
        "requestIntegrityToken",
        "(Lcom/google/android/play/core/integrity/IntegrityTokenRequest;)Lcom/google/android/gms/tasks/Task;",
        manager,
        [Value::Obj(request)]
    )
    .unwrap())
}

fn manager(mut e: &mut Engine) -> u32 {
    let c = ctx(e);
    ov(cs!(
        e,
        INTEGRITY_FACTORY,
        "create",
        "(Landroid/content/Context;)Lcom/google/android/play/core/integrity/IntegrityManager;",
        [c]
    )
    .unwrap())
}

/// Mensagem da exceção (campo `message`, como o runtime grava em `make_exception`).
fn exc_message(e: &Engine, exc: u32) -> String {
    match e.heap.get_field(exc, "message") {
        Ok(Value::Obj(m)) => e.heap.as_str(m).unwrap().to_string(),
        other => panic!("mensagem ausente: {other:?}"),
    }
}

fn task_flag(mut e: &mut Engine, task: u32, name: &str) -> i32 {
    iv(ci!(e, TASK, name, "()Z", task, []).unwrap())
}

#[test]
fn integrity_failure_then_recovery_is_a_full_lifecycle() {
    let mut e = engine();
    let m = manager(&mut e);

    // 1) Sucesso: token padrão, nonce registrado.
    let t1 = request_token(&mut e, m, "n1");
    assert_eq!(task_flag(&mut e, t1, "isSuccessful"), 1, "sucesso inicial");

    // 2) Erro programado: API não disponível (-1). A Task falha com a exceção da lib.
    e.gms_set_integrity_failure(-1);
    let t2 = request_token(&mut e, m, "n2");
    assert_eq!(task_flag(&mut e, t2, "isSuccessful"), 0, "falha programada");
    assert_eq!(
        task_flag(&mut e, t2, "isComplete"),
        1,
        "falha também completa a Task"
    );
    let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t2, []).unwrap());
    let class = e.heap.class_of(exc).unwrap().to_string();
    assert_eq!(class, INTEGRITY_SERVICE_EXCEPTION, "tipo da exceção da lib");
    assert_eq!(exc_message(&e, exc), "Integrity API error (-1)");
    assert_eq!(
        e.gms_integrity_nonces(),
        vec!["integrity:n1".to_string()],
        "pedido que falhou não registra nonce"
    );

    // 3) Recuperação: limpar a falha volta ao sucesso, sem resíduo do erro.
    e.gms_clear_integrity_failure();
    let t3 = request_token(&mut e, m, "n3");
    assert_eq!(
        task_flag(&mut e, t3, "isSuccessful"),
        1,
        "recuperado após limpar a falha"
    );
    let resp = ov(ci!(e, TASK, "getResult", "()Ljava/lang/Object;", t3, []).unwrap());
    let tok_v = ci!(
        e,
        INTEGRITY_TOKEN_RESPONSE,
        "token",
        "()Ljava/lang/String;",
        resp,
        []
    )
    .unwrap();
    let tok = sv(&e, tok_v);
    assert_eq!(tok, "rustydroid-fake-integrity-token");
    assert_eq!(
        e.gms_integrity_nonces(),
        vec!["integrity:n1".to_string(), "integrity:n3".to_string()],
        "só os pedidos aceitos registram nonce"
    );
}

#[test]
fn integrity_failure_code_is_programmable_per_error_kind() {
    // Códigos de IntegrityErrorCode (rede -3, erro interno -100): a mensagem acompanha o código.
    let mut e = engine();
    let m = manager(&mut e);
    for code in [-3, -100] {
        e.gms_set_integrity_failure(code);
        let t = request_token(&mut e, m, "n");
        assert_eq!(task_flag(&mut e, t, "isSuccessful"), 0, "código {code}");
        let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t, []).unwrap());
        assert_eq!(
            exc_message(&e, exc),
            format!("Integrity API error ({code})")
        );
    }
    assert!(
        e.gms_integrity_nonces().is_empty(),
        "nenhum pedido chegou ao serviço"
    );
}

const HISTORY: &str = "Lcom/google/android/gms/fitness/HistoryClient;";
const SESSIONS: &str = "Lcom/google/android/gms/fitness/SessionsClient;";
const FIT_CTX_SIG: &str =
    "(Landroid/content/Context;Lcom/google/android/gms/auth/api/signin/GoogleSignInAccount;)";

fn fit_client(mut e: &mut Engine, getter: &str, class: &str) -> u32 {
    let c = ctx(e);
    ov(cs!(
        e,
        FITNESS,
        getter,
        &format!("{FIT_CTX_SIG}{class})"),
        [c, Value::Null]
    )
    .unwrap())
}

fn read_daily(mut e: &mut Engine, history: u32) -> u32 {
    ov(ci!(
        e,
        HISTORY,
        "readDailyTotal",
        "(Lcom/google/android/gms/fitness/data/DataType;)Lcom/google/android/gms/tasks/Task;",
        history,
        [Value::Null]
    )
    .unwrap())
}

fn insert_data(mut e: &mut Engine, history: u32) -> u32 {
    ov(ci!(
        e,
        HISTORY,
        "insertData",
        "(Lcom/google/android/gms/fitness/data/DataSet;)Lcom/google/android/gms/tasks/Task;",
        history,
        [Value::Null]
    )
    .unwrap())
}

/// Passos do DataSet resultante de `readDailyTotal` (mesmo caminho do teste de Fit existente).
fn steps_of(mut e: &mut Engine, task: u32) -> i32 {
    let data_set = ov(ci!(e, TASK, "getResult", "()Ljava/lang/Object;", task, []).unwrap());
    let points = ov(ci!(
        e,
        DATA_SET,
        "getDataPoints",
        "()Ljava/util/List;",
        data_set,
        []
    )
    .unwrap());
    let point = match e.fw.objects.get(&points).cloned() {
        Some(rd_vm::framework::HostObj::List { elems }) => match elems.first() {
            Some(Value::Obj(r)) => *r,
            other => panic!("datapoint: {other:?}"),
        },
        other => panic!("points: {other:?}"),
    };
    let field = obj(e, "Lcom/google/android/gms/fitness/data/Field;");
    let value = ov(ci!(
        e,
        DATA_POINT,
        "getValue",
        "(Lcom/google/android/gms/fitness/data/Field;)Lcom/google/android/gms/fitness/data/Value;",
        point,
        [Value::Obj(field)]
    )
    .unwrap());
    iv(ci!(e, FIT_VALUE, "asInt", "()I", value, []).unwrap())
}

#[test]
fn fit_failure_blocks_reads_and_writes_then_recovers() {
    let mut e = engine();
    e.gms_fit_set_steps(4321);
    let history = fit_client(&mut e, "getHistoryClient", HISTORY);

    // 1) Sucesso: leitura devolve os passos programados.
    let t1 = read_daily(&mut e, history);
    assert_eq!(task_flag(&mut e, t1, "isSuccessful"), 1, "leitura inicial");
    assert_eq!(steps_of(&mut e, t1), 4321);

    // 2) Erro: leitura e escrita falham com ApiException e a escrita não grava.
    e.gms_fit_set_failure(7);
    let t2 = read_daily(&mut e, history);
    assert_eq!(task_flag(&mut e, t2, "isSuccessful"), 0, "leitura falha");
    let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t2, []).unwrap());
    assert_eq!(e.heap.class_of(exc).unwrap(), API_EXCEPTION);
    assert_eq!(exc_message(&e, exc), "Fitness API error (7)");
    let w = insert_data(&mut e, history);
    assert_eq!(task_flag(&mut e, w, "isSuccessful"), 0, "escrita falha");
    assert_eq!(e.fw.gms.fit_inserts, 0, "falha não grava dados");

    // 3) Recuperação: sem resíduo, leitura e escrita voltam a funcionar.
    e.gms_fit_clear_failure();
    let t3 = read_daily(&mut e, history);
    assert_eq!(steps_of(&mut e, t3), 4321, "leitura após limpar a falha");
    let w2 = insert_data(&mut e, history);
    assert_eq!(
        task_flag(&mut e, w2, "isSuccessful"),
        1,
        "escrita após limpar a falha"
    );
    assert_eq!(e.fw.gms.fit_inserts, 1, "exatamente uma gravação");
}

#[test]
fn fit_failure_does_not_start_sessions() {
    let mut e = engine();
    let sessions = fit_client(&mut e, "getSessionsClient", SESSIONS);
    let session = ctx(&mut e);

    e.gms_fit_set_failure(3);
    let t = ov(ci!(
        e,
        SESSIONS,
        "startSession",
        "(Lcom/google/android/gms/fitness/request/SessionInsertRequest;)Lcom/google/android/gms/tasks/Task;",
        sessions,
        [session.clone()]
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t, "isSuccessful"), 0, "sessão falha");
    assert!(
        e.fw.gms.fit_sessions_started.is_empty(),
        "sessão falha não inicia"
    );

    e.gms_fit_clear_failure();
    let t2 = ov(ci!(
        e,
        SESSIONS,
        "startSession",
        "(Lcom/google/android/gms/fitness/request/SessionInsertRequest;)Lcom/google/android/gms/tasks/Task;",
        sessions,
        [session]
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, t2, "isSuccessful"),
        1,
        "sessão após limpar a falha"
    );
    assert_eq!(e.fw.gms.fit_sessions_started.len(), 1);
}

const LOC_CB: &str = "LLocErroCb;";

/// Classe DEX cujo callback incrementa `called` (prova a entrega por bytecode do convidado).
fn loc_callback_class(b: &mut DexBuilder) {
    let cls = b.class(LOC_CB, "Ljava/lang/Object;");
    b.static_field(cls, "called", "I", common::SVal::Int(0));
    let f = b.field_idx(LOC_CB, "I", "called");
    let params = vec!["Lcom/google/android/gms/location/LocationResult;"];
    b.r#virtual(
        cls,
        "onLocationResult",
        "V",
        params,
        ACC_PUBLIC,
        Some(b.code(3, 2, 0, {
            let mut u = op21c(0x60, 0, f);
            u.extend(op11n(0x12, 1, 1));
            u.extend(op23x(0x90, 0, 0, 1));
            u.extend(op21c(0x67, 0, f));
            u.extend(op10x(0x0E));
            u
        })),
    );
}

fn called_count(e: &Engine) -> i32 {
    match e.statics.get(&(LOC_CB.to_string(), "called".to_string())) {
        Some(Value::Int(n)) => *n,
        None => 0,
        other => panic!("called: {other:?}"),
    }
}

#[test]
fn location_failure_blocks_reads_and_registrations_then_recovers() {
    let mut b = DexBuilder::new();
    loc_callback_class(&mut b);
    let mut e = engine_of(&b);
    let c = ctx(&mut e);
    let fused = ov(cs!(
        e,
        LOCATION_SERVICES,
        "getFusedLocationProviderClient",
        "(Landroid/content/Context;)Lcom/google/android/gms/location/FusedLocationProviderClient;",
        [c]
    )
    .unwrap());
    e.gms_set_location_fix(-23.55, -46.63, 8.0);

    // 1) Sucesso: última posição e registro entregam a posição.
    let t = ov(ci!(
        e,
        FUSED_CLIENT,
        "getLastLocation",
        "()Lcom/google/android/gms/tasks/Task;",
        fused,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t, "isSuccessful"), 1, "leitura inicial");

    // 2) Erro: leitura falha com ApiException; o registro falha e não guarda callback.
    e.gms_location_set_failure(8);
    let t2 = ov(ci!(
        e,
        FUSED_CLIENT,
        "getLastLocation",
        "()Lcom/google/android/gms/tasks/Task;",
        fused,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t2, "isSuccessful"), 0, "leitura falha");
    let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t2, []).unwrap());
    assert_eq!(e.heap.class_of(exc).unwrap(), API_EXCEPTION);
    assert_eq!(exc_message(&e, exc), "Location API error (8)");

    let req = ov(cs!(
        e,
        LOCATION_REQUEST,
        "create",
        "()Lcom/google/android/gms/location/LocationRequest;",
        []
    )
    .unwrap());
    let cb = obj(&mut e, LOC_CB);
    let t3 = ov(ci!(
        e,
        FUSED_CLIENT,
        "requestLocationUpdates",
        "(Lcom/google/android/gms/location/LocationRequest;Lcom/google/android/gms/location/LocationCallback;Landroid/os/Looper;)Lcom/google/android/gms/tasks/Task;",
        fused,
        [Value::Obj(req), Value::Obj(cb), Value::Null]
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t3, "isSuccessful"), 0, "registro falha");
    assert_eq!(
        e.gms_location_callback_count(),
        0,
        "falha não registra callback"
    );
    assert_eq!(called_count(&e), 0, "falha não entrega posição");

    // 3) Recuperação: o registro volta a entregar e a remoção desfaz o registro.
    e.gms_location_clear_failure();
    let t4 = ov(ci!(
        e,
        FUSED_CLIENT,
        "requestLocationUpdates",
        "(Lcom/google/android/gms/location/LocationRequest;Lcom/google/android/gms/location/LocationCallback;Landroid/os/Looper;)Lcom/google/android/gms/tasks/Task;",
        fused,
        [Value::Obj(req), Value::Obj(cb), Value::Null]
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, t4, "isSuccessful"),
        1,
        "registro após limpar a falha"
    );
    assert_eq!(called_count(&e), 1, "entrega inicial do registro");
    assert_eq!(e.gms_location_callback_count(), 1);

    // Remover não depende do serviço: mesmo com falha programada, desregistra.
    e.gms_location_set_failure(8);
    let t5 = ov(ci!(
        e,
        FUSED_CLIENT,
        "removeLocationUpdates",
        "(Lcom/google/android/gms/location/LocationCallback;)Lcom/google/android/gms/tasks/Task;",
        fused,
        [Value::Obj(cb)]
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, t5, "isSuccessful"),
        1,
        "remoção funciona com falha programada"
    );
    assert_eq!(
        e.gms_location_callback_count(),
        0,
        "remoção desfaz o registro"
    );
}
