//! Apêndice E — critério de erro e ciclo de vida (`gms_erro_ciclo.rs`).
//!
//! Serviços com caminho de falha programável: a Task falha com exceção do
//! tipo da biblioteca, o estado de erro não vaza para o pedido seguinte e a
//! recuperação (limpar a falha) volta ao sucesso. Nenhuma rede: o estado é
//! programado pelo hospedeiro (`Engine::gms_*`) e lido pelo convidado.

#[macro_use]
mod common;

use common::*;
use rd_vm::framework::HostObj;
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

fn rc_string(mut e: &mut Engine, rc: u32, key: &str) -> String {
    let k = s(e, key);
    let v = ci!(
        e,
        REMOTE_CONFIG,
        "getString",
        "(Ljava/lang/String;)Ljava/lang/String;",
        rc,
        [k]
    )
    .unwrap();
    sv(e, v)
}

#[test]
fn remote_config_failed_fetch_keeps_active_values_then_recovers() {
    let mut e = engine();
    let rc = ov(cs!(
        e,
        REMOTE_CONFIG,
        "getInstance",
        "()Lcom/google/firebase/RemoteConfig/FirebaseRemoteConfig;",
        []
    )
    .unwrap());

    // 1) Sucesso: fetchAndActivate aplica o valor servido.
    e.gms_remoteconfig_program(vec![("banner".into(), "v1".into())]);
    let t1 = ov(ci!(
        e,
        REMOTE_CONFIG,
        "fetchAndActivate",
        "()Lcom/google/android/gms/tasks/Task;",
        rc,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t1, "isSuccessful"), 1, "fetch inicial");
    assert_eq!(rc_string(&mut e, rc, "banner"), "v1");

    // 2) Erro no fetch: o servidor tem v2, mas a falha impede a troca; v1 continua ativo.
    e.gms_remoteconfig_program(vec![("banner".into(), "v2".into())]);
    e.gms_remoteconfig_set_fetch_failure(1);
    let t2 = ov(ci!(
        e,
        REMOTE_CONFIG,
        "fetchAndActivate",
        "()Lcom/google/android/gms/tasks/Task;",
        rc,
        []
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, t2, "isSuccessful"),
        0,
        "fetchAndActivate falha"
    );
    let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t2, []).unwrap());
    assert_eq!(e.heap.class_of(exc).unwrap(), RC_FETCH_EXCEPTION);
    assert_eq!(
        rc_string(&mut e, rc, "banner"),
        "v1",
        "valor ativo preservado após falha"
    );

    // Caminho separado fetch + activate: fetch falho não deixa nada pendente para ativar.
    let t3 = ov(ci!(
        e,
        REMOTE_CONFIG,
        "fetch",
        "()Lcom/google/android/gms/tasks/Task;",
        rc,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t3, "isSuccessful"), 0, "fetch falha");
    let _ = ci!(
        e,
        REMOTE_CONFIG,
        "activate",
        "()Lcom/google/android/gms/tasks/Task;",
        rc,
        []
    );
    assert_eq!(
        rc_string(&mut e, rc, "banner"),
        "v1",
        "activate sem fetch válido não troca"
    );

    // 3) Recuperação: sem falha, o próximo fetchAndActivate aplica v2.
    e.gms_remoteconfig_clear_fetch_failure();
    let t4 = ov(ci!(
        e,
        REMOTE_CONFIG,
        "fetchAndActivate",
        "()Lcom/google/android/gms/tasks/Task;",
        rc,
        []
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, t4, "isSuccessful"),
        1,
        "fetch após recuperação"
    );
    assert_eq!(
        rc_string(&mut e, rc, "banner"),
        "v2",
        "novo valor após recuperação"
    );
}

fn cast_context(mut e: &mut Engine) -> u32 {
    let c = ctx(e);
    ov(cs!(
        e,
        CAST_CONTEXT,
        "getSharedInstance",
        "(Landroid/content/Context;)Lcom/google/android/gms/cast/framework/CastContext;",
        [c]
    )
    .unwrap())
}

#[test]
fn cast_failed_connect_creates_no_session_then_recovers() {
    let mut e = engine();
    let cc = cast_context(&mut e);
    let state =
        |mut e: &mut Engine| iv(ci!(e, CAST_CONTEXT, "getCastState", "()I", cc, []).unwrap());

    // 1) Sucesso: conecta e expõe a sessão ao convidado.
    e.gms_cast_connect("TV da Sala");
    assert_eq!(state(&mut e), 4, "CONNECTED");
    assert_eq!(e.gms_cast_last_error(), None);
    e.gms_cast_disconnect();
    assert_eq!(state(&mut e), 1, "volta a NO_DEVICES");

    // 2) Erro: a tentativa falha; sem sessão e com o erro registrado.
    e.gms_cast_set_connect_failure(2);
    e.gms_cast_connect("TV do Quarto");
    assert_eq!(state(&mut e), 1, "falha não conecta");
    assert_eq!(e.gms_cast_last_error(), Some(2), "erro registrado");
    let sm = ov(ci!(
        e,
        CAST_CONTEXT,
        "getSessionManager",
        "()Lcom/google/android/gms/cast/framework/SessionManager;",
        cc,
        []
    )
    .unwrap());
    let session = ci!(
        e,
        SESSION_MANAGER,
        "getCurrentCastSession",
        "()Lcom/google/android/gms/cast/framework/CastSession;",
        sm,
        []
    )
    .unwrap();
    assert!(
        matches!(session, Some(Value::Null)),
        "sem sessão após falha: {session:?}"
    );

    // 3) Recuperação: sem falha, a nova tentativa conecta e limpa o erro.
    e.gms_cast_clear_connect_failure();
    e.gms_cast_connect("TV do Quarto");
    assert_eq!(state(&mut e), 4, "conecta após limpar a falha");
    assert_eq!(
        e.gms_cast_last_error(),
        None,
        "erro limpo pela tentativa bem-sucedida"
    );
}

// ── Analytics: nome inválido pela regra documentada é rejeitado e contado;
// nomes válidos seguem gravando; coleção desligada não conta como rejeição.
fn bundle_of(e: &mut Engine, entries: Vec<(String, Value)>) -> Value {
    let r = obj(e, "Landroid/os/Bundle;");
    e.fw.objects.insert(r, HostObj::Bundle { map: entries });
    Value::Obj(r)
}

fn log_event(mut e: &mut Engine, an: u32, name: &str, params: Value) {
    let n = s(e, name);
    ci!(
        e,
        FIREBASE_ANALYTICS,
        "logEvent",
        "(Ljava/lang/String;Landroid/os/Bundle;)V",
        an,
        [n, params]
    )
    .unwrap();
}

#[test]
fn analytics_invalid_names_are_rejected_then_valid_events_recorded() {
    let mut e = engine();
    let an = ov(cs!(
        e,
        FIREBASE_ANALYTICS,
        "getInstance",
        "(Landroid/content/Context;)Lcom/google/firebase/analytics/FirebaseAnalytics;",
        [ctx(&mut e)]
    )
    .unwrap());

    // Evento: começa com dígito, tem hífen, usa prefixo reservado, passa de 40.
    log_event(&mut e, an, "1_inicio", Value::Null);
    log_event(&mut e, an, "bad-name", Value::Null);
    log_event(&mut e, an, "firebase_evento", Value::Null);
    let longo = "a".repeat(41);
    log_event(&mut e, an, &longo, Value::Null);
    assert_eq!(e.gms_analytics_rejected(), 4);
    assert!(e.gms_analytics_events().is_empty(), "nada gravado");

    // Parâmetro inválido rejeita o evento inteiro.
    let bad_param = {
        let v = s(&mut e, "1");
        bundle_of(&mut e, vec![("preço-x".to_string(), v)])
    };
    log_event(&mut e, an, "compra", bad_param);
    assert_eq!(e.gms_analytics_rejected(), 5);
    assert!(e.gms_analytics_events().is_empty());

    // Propriedade de usuário com nome inválido (máximo 24).
    let k = s(&mut e, "1plano");
    let v = s(&mut e, "pro");
    ci!(
        e,
        FIREBASE_ANALYTICS,
        "setUserProperty",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        an,
        [k, v]
    )
    .unwrap();
    assert_eq!(e.gms_analytics_rejected(), 6);
    assert!(e.gms_analytics_user_props().is_empty());

    // Nomes válidos (inclusive com 40 caracteres) são gravados.
    let item = {
        let v = s(&mut e, "banana");
        bundle_of(&mut e, vec![("item".to_string(), v)])
    };
    log_event(&mut e, an, "compra_ok", item);
    let limite = "b".repeat(40);
    log_event(&mut e, an, &limite, Value::Null);
    assert_eq!(e.gms_analytics_rejected(), 6, "válidos não contam");
    let nomes: Vec<&str> = e
        .gms_analytics_events()
        .iter()
        .map(|ev| ev.name.as_str())
        .collect();
    assert_eq!(nomes, vec!["compra_ok", limite.as_str()]);

    // Ciclo: coleção desligada descarta sem contar como rejeição.
    ci!(
        e,
        FIREBASE_ANALYTICS,
        "setAnalyticsCollectionEnabled",
        "(Z)V",
        an,
        [Value::Int(0)]
    )
    .unwrap();
    log_event(&mut e, an, "oculto", Value::Null);
    assert_eq!(e.gms_analytics_events().len(), 2);
    assert_eq!(e.gms_analytics_rejected(), 6);

    // Religar a coleta volta a gravar.
    ci!(
        e,
        FIREBASE_ANALYTICS,
        "setAnalyticsCollectionEnabled",
        "(Z)V",
        an,
        [Value::Int(1)]
    )
    .unwrap();
    log_event(&mut e, an, "volta", Value::Null);
    assert_eq!(e.gms_analytics_events().len(), 3);
    assert_eq!(e.gms_analytics_events()[2].name, "volta");
}

// ── Crashlytics: no máximo 64 pares chave/valor (novas chaves além do limite
// são ignoradas; regravar uma chave existente continua valendo) e chave/valor
// acima de 1024 caracteres são truncados. Vale para setCustomKey e setCustomKeys.
fn crash_key(mut e: &mut Engine, cl: u32, k: &str, v: &str) {
    let kk = s(e, k);
    let vv = s(e, v);
    ci!(
        e,
        FIREBASE_CRASHLYTICS,
        "setCustomKey",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        cl,
        [kk, vv]
    )
    .unwrap();
}

#[test]
fn crashlytics_custom_keys_capped_at_64_and_truncated() {
    let mut e = engine();
    let cl = ov(cs!(
        e,
        FIREBASE_CRASHLYTICS,
        "getInstance",
        "()Lcom/google/firebase/crashlytics/FirebaseCrashlytics;",
        []
    )
    .unwrap());

    // 70 chaves individuais: só as 64 primeiras entram.
    for i in 0..70 {
        crash_key(&mut e, cl, &format!("k{i}"), "v");
    }
    let keys = e.gms_crashlytics_keys();
    assert_eq!(keys.len(), 64, "limite de 64 pares");
    assert_eq!(keys.last().unwrap().0, "k63", "k64..k69 ignoradas");

    // Regravar chave existente no limite atualiza o valor.
    crash_key(&mut e, cl, "k0", "novo");
    let keys = e.gms_crashlytics_keys();
    assert_eq!(keys.len(), 64);
    assert_eq!(keys[0], ("k0".to_string(), "novo".to_string()));

    // setCustomKeys (CustomKeysAndValues) também respeita o limite.
    let b = obj(&mut e, CKAV_BUILDER);
    ci!(e, CKAV_BUILDER, "<init>", "()V", b, []).unwrap();
    let k = s(&mut e, "nova_ckav");
    let v = s(&mut e, "x");
    ci!(
        e,
        CKAV_BUILDER,
        "put",
        "(Ljava/lang/String;Ljava/lang/String;)Lcom/google/firebase/crashlytics/CustomKeysAndValues$Builder;",
        b,
        [k, v]
    )
    .unwrap();
    let ckav = ov(ci!(
        e,
        CKAV_BUILDER,
        "build",
        "()Lcom/google/firebase/crashlytics/CustomKeysAndValues;",
        b,
        []
    )
    .unwrap());
    ci!(
        e,
        FIREBASE_CRASHLYTICS,
        "setCustomKeys",
        "(Lcom/google/firebase/crashlytics/CustomKeysAndValues;)V",
        cl,
        [Value::Obj(ckav)]
    )
    .unwrap();
    let keys = e.gms_crashlytics_keys();
    assert_eq!(keys.len(), 64, "setCustomKeys não ultrapassa 64");
    assert!(!keys.iter().any(|(k, _)| k == "nova_ckav"));

    // Truncamento: chave e valor de 1500 caracteres ficam com 1024.
    let mut e2 = engine();
    let cl2 = ov(cs!(
        e2,
        FIREBASE_CRASHLYTICS,
        "getInstance",
        "()Lcom/google/firebase/crashlytics/FirebaseCrashlytics;",
        []
    )
    .unwrap());
    let longa = "x".repeat(1500);
    crash_key(&mut e2, cl2, &longa, &"z".repeat(1500));
    let keys = e2.gms_crashlytics_keys();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].0.chars().count(), 1024);
    assert_eq!(keys[0].1.chars().count(), 1024);
}

// ── Maps: falha de carregamento programada. `MapsInitializer.initialize`
// devolve o código de erro e `getMapAsync` não entrega o GoogleMap enquanto a
// falha durar; limpar a falha volta ao carregamento normal.
const MAP_CB: &str = "LMapReadyCb;";

fn map_ready_class(b: &mut DexBuilder) {
    let cls = b.class(MAP_CB, "Ljava/lang/Object;");
    b.static_field(cls, "called", "I", common::SVal::Int(0));
    let f = b.field_idx(MAP_CB, "I", "called");
    let params = vec!["Lcom/google/android/gms/maps/GoogleMap;"];
    b.r#virtual(
        cls,
        "onMapReady",
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

fn map_ready_count(e: &Engine) -> i32 {
    match e.statics.get(&(MAP_CB.to_string(), "called".to_string())) {
        Some(Value::Int(n)) => *n,
        None => 0,
        other => panic!("called: {other:?}"),
    }
}

#[test]
fn maps_load_failure_blocks_map_ready_then_recovers() {
    let mut b = DexBuilder::new();
    map_ready_class(&mut b);
    let mut e = engine_of(&b);
    let frag = ov(cs!(
        e,
        SUPPORT_MAP_FRAGMENT,
        "newInstance",
        "()Lcom/google/android/gms/maps/SupportMapFragment;",
        []
    )
    .unwrap());
    let cb = obj(&mut e, MAP_CB);

    // Falha programada: initialize devolve o código e o mapa não é entregue.
    e.gms_maps_set_load_failure(1);
    let code = iv(cs!(
        e,
        MAPS_INITIALIZER,
        "initialize",
        "(Landroid/content/Context;)I",
        [ctx(&mut e)]
    )
    .unwrap());
    assert_eq!(code, 1, "initialize devolve o código de falha");
    ci!(
        e,
        SUPPORT_MAP_FRAGMENT,
        "getMapAsync",
        "(Lcom/google/android/gms/maps/OnMapReadyCallback;)V",
        frag,
        [Value::Obj(cb)]
    )
    .unwrap();
    assert_eq!(map_ready_count(&e), 0, "sem mapa, sem onMapReady");

    // Limpar a falha: initialize volta a SUCCESS e o mapa é entregue.
    e.gms_maps_clear_load_failure();
    let code = iv(cs!(
        e,
        MAPS_INITIALIZER,
        "initialize",
        "(Landroid/content/Context;)I",
        [ctx(&mut e)]
    )
    .unwrap());
    assert_eq!(code, 0, "SUCCESS após limpar a falha");
    ci!(
        e,
        SUPPORT_MAP_FRAGMENT,
        "getMapAsync",
        "(Lcom/google/android/gms/maps/OnMapReadyCallback;)V",
        frag,
        [Value::Obj(cb)]
    )
    .unwrap();
    assert_eq!(map_ready_count(&e), 1, "mapa entregue após recuperação");
}

// ── Messaging (FCM): falha programada no token e nos tópicos. A Task falha com
// java.io.IOException (SERVICE_NOT_AVAILABLE) e não muda estado; deleteToken
// segue disponível; limpar a falha volta ao sucesso.
#[test]
fn messaging_failure_blocks_token_and_topics_then_recovers() {
    let mut e = engine();
    let msg = ov(cs!(
        e,
        FIREBASE_MESSAGING,
        "getInstance",
        "()Lcom/google/firebase/messaging/FirebaseMessaging;",
        []
    )
    .unwrap());
    e.gms_set_fcm_token("tok-1");

    // Token falha com IOException e não devolve valor.
    e.gms_fcm_set_failure();
    let t = ov(ci!(
        e,
        FIREBASE_MESSAGING,
        "getToken",
        "()Lcom/google/android/gms/tasks/Task;",
        msg,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t, "isSuccessful"), 0, "token falha");
    assert_eq!(
        task_flag(&mut e, t, "isComplete"),
        1,
        "falha também completa a Task"
    );
    let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t, []).unwrap());
    let class = e.heap.class_of(exc).unwrap().to_string();
    assert_eq!(class, "Ljava/io/IOException;", "tipo da exceção");
    assert_eq!(exc_message(&e, exc), "SERVICE_NOT_AVAILABLE");

    // Tópico: a assinatura falha e o tópico não entra.
    let topic = s(&mut e, "noticias");
    let sub = ov(ci!(
        e,
        FIREBASE_MESSAGING,
        "subscribeToTopic",
        "(Ljava/lang/String;)Lcom/google/android/gms/tasks/Task;",
        msg,
        [topic]
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, sub, "isSuccessful"),
        0,
        "assinatura falha"
    );
    assert!(e.gms_fcm_topics().is_empty(), "tópico não entra");

    // deleteToken não depende do serviço neste modelo: segue com sucesso.
    let del = ov(ci!(
        e,
        FIREBASE_MESSAGING,
        "deleteToken",
        "()Lcom/google/android/gms/tasks/Task;",
        msg,
        []
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, del, "isSuccessful"),
        1,
        "deleteToken segue"
    );

    // Recuperação: token e assinatura voltam a funcionar.
    e.gms_fcm_clear_failure();
    let topic = s(&mut e, "noticias");
    let t = ov(ci!(
        e,
        FIREBASE_MESSAGING,
        "getToken",
        "()Lcom/google/android/gms/tasks/Task;",
        msg,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t, "isSuccessful"), 1, "token volta");
    let sub = ov(ci!(
        e,
        FIREBASE_MESSAGING,
        "subscribeToTopic",
        "(Ljava/lang/String;)Lcom/google/android/gms/tasks/Task;",
        msg,
        [topic]
    )
    .unwrap());
    assert_eq!(
        task_flag(&mut e, sub, "isSuccessful"),
        1,
        "assinatura volta"
    );
    assert_eq!(e.gms_fcm_topics(), vec!["noticias".to_string()]);
}

// ── RTDB: negação das regras. Leituras por listener chamam onCancelled e o
// listener não fica registrado; escritas não acontecem; removeValue, updateChildren
// e get falham. Limpar a negação volta ao comportamento normal.
const RTDB_CB: &str = "LRtdbCb;";

fn rtdb_cb_class(b: &mut DexBuilder) {
    let cls = b.class(RTDB_CB, "Ljava/lang/Object;");
    b.static_field(cls, "data", "I", common::SVal::Int(0));
    b.static_field(cls, "cancelled", "I", common::SVal::Int(0));
    for (method, param, field) in [
        (
            "onDataChange",
            "Lcom/google/firebase/database/DataSnapshot;",
            "data",
        ),
        (
            "onCancelled",
            "Lcom/google/firebase/database/DatabaseError;",
            "cancelled",
        ),
    ] {
        let f = b.field_idx(RTDB_CB, "I", field);
        b.r#virtual(
            cls,
            method,
            "V",
            vec![param],
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
}

fn rtdb_count(e: &Engine, field: &str) -> i32 {
    match e.statics.get(&(RTDB_CB.to_string(), field.to_string())) {
        Some(Value::Int(n)) => *n,
        None => 0,
        other => panic!("{field}: {other:?}"),
    }
}

#[test]
fn rtdb_permission_denied_cancels_listeners_and_blocks_writes_then_recovers() {
    let mut b = DexBuilder::new();
    rtdb_cb_class(&mut b);
    let mut e = engine_of(&b);
    let db = ov(cs!(
        e,
        FIREBASE_DATABASE,
        "getInstance",
        "()Lcom/google/firebase/database/FirebaseDatabase;",
        []
    )
    .unwrap());
    let path = s(&mut e, "config/tema");
    let reference = ov(ci!(
        e,
        FIREBASE_DATABASE,
        "getReference",
        "(Ljava/lang/String;)Lcom/google/firebase/database/DatabaseReference;",
        db,
        [path]
    )
    .unwrap());
    let v1 = s(&mut e, "escuro");
    ci!(
        e,
        DATABASE_REF,
        "setValue",
        "(Ljava/lang/Object;)V",
        reference,
        [v1]
    )
    .unwrap();

    // Sem negação: o listener recebe o valor ao registrar.
    let l1 = obj(&mut e, RTDB_CB);
    ci!(
        e,
        DATABASE_REF,
        "addValueEventListener",
        "(Lcom/google/firebase/database/ValueEventListener;)Lcom/google/firebase/database/ValueEventListener;",
        reference,
        [Value::Obj(l1)]
    )
    .unwrap();
    assert_eq!(rtdb_count(&e, "data"), 1, "entrega normal ao registrar");

    // Negação: escrita não acontece e remoção/leitura falham.
    e.gms_rtdb_set_permission_denied(true);
    let v2 = s(&mut e, "claro");
    ci!(
        e,
        DATABASE_REF,
        "setValue",
        "(Ljava/lang/Object;)V",
        reference,
        [v2]
    )
    .unwrap();
    assert_eq!(
        sv(&e, e.fw.gms.rtdb_values.get("config/tema").cloned()),
        "escuro",
        "escrita negada não altera o valor"
    );
    assert_eq!(
        rtdb_count(&e, "data"),
        1,
        "nenhuma entrega com acesso negado"
    );
    let rm = ov(ci!(
        e,
        DATABASE_REF,
        "removeValue",
        "()Lcom/google/android/gms/tasks/Task;",
        reference,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, rm, "isSuccessful"), 0, "remoção falha");
    let g = ov(ci!(
        e,
        DATABASE_REF,
        "get",
        "()Lcom/google/android/gms/tasks/Task;",
        reference,
        []
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, g, "isSuccessful"), 0, "leitura falha");

    // Novo listener sob negação é cancelado (sem onDataChange) e não fica registrado.
    let l2 = obj(&mut e, RTDB_CB);
    ci!(
        e,
        DATABASE_REF,
        "addValueEventListener",
        "(Lcom/google/firebase/database/ValueEventListener;)Lcom/google/firebase/database/ValueEventListener;",
        reference,
        [Value::Obj(l2)]
    )
    .unwrap();
    assert_eq!(
        rtdb_count(&e, "cancelled"),
        1,
        "cancelado com PERMISSION_DENIED"
    );
    assert_eq!(rtdb_count(&e, "data"), 1, "sem entrega de dados");

    // Recuperação: a escrita volta a valer e o listener registrado recebe a mudança.
    e.gms_rtdb_set_permission_denied(false);
    let v3 = s(&mut e, "novo");
    ci!(
        e,
        DATABASE_REF,
        "setValue",
        "(Ljava/lang/Object;)V",
        reference,
        [v3]
    )
    .unwrap();
    assert_eq!(
        sv(&e, e.fw.gms.rtdb_values.get("config/tema").cloned()),
        "novo"
    );
    assert_eq!(
        rtdb_count(&e, "data"),
        2,
        "listener registrado volta a receber"
    );
    assert_eq!(rtdb_count(&e, "cancelled"), 1, "nenhum cancelamento extra");
}

// ── SafetyNet: falha programada no attest. A Task falha com ApiException, o
// pedido falho não registra nonce; limpar a falha volta ao sucesso.
#[test]
fn safetynet_failure_blocks_attestation_then_recovers() {
    let mut e = engine();
    let sn = ov(cs!(
        e,
        SAFETYNET,
        "getClient",
        "(Landroid/content/Context;)Lcom/google/android/gms/safetynet/SafetyNetClient;",
        [ctx(&mut e)]
    )
    .unwrap());

    e.gms_safetynet_set_failure(7);
    let key = s(&mut e, "api-key");
    let t = ov(ci!(
        e,
        SAFETYNET_CLIENT,
        "attest",
        "([BLjava/lang/String;)Lcom/google/android/gms/tasks/Task;",
        sn,
        [Value::Null, key]
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t, "isSuccessful"), 0, "attest falha");
    let exc = ov(ci!(e, TASK, "getException", "()Ljava/lang/Exception;", t, []).unwrap());
    let class = e.heap.class_of(exc).unwrap().to_string();
    assert_eq!(class, API_EXCEPTION, "tipo da exceção da lib");
    assert_eq!(exc_message(&e, exc), "SafetyNet API error (7)");
    assert!(
        e.gms_integrity_nonces().is_empty(),
        "pedido falho não registra nonce"
    );

    // Recuperação: attest volta a concluir e registra o nonce.
    e.gms_safetynet_clear_failure();
    let key2 = s(&mut e, "api-key");
    let t = ov(ci!(
        e,
        SAFETYNET_CLIENT,
        "attest",
        "([BLjava/lang/String;)Lcom/google/android/gms/tasks/Task;",
        sn,
        [Value::Null, key2]
    )
    .unwrap());
    assert_eq!(task_flag(&mut e, t, "isSuccessful"), 1, "attest volta");
    let nonces = e.gms_integrity_nonces();
    assert_eq!(nonces.len(), 1);
    assert!(
        nonces[0].starts_with("safetynet:"),
        "nonce registrado: {nonces:?}"
    );
}
