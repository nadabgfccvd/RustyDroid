//! Apêndice D — `androidx.credentials` (host-shim sem provedor).
//!
//! Um DEX sintético chama `CredentialManager` por `invoke-interface/range`,
//! como um app faria, e o callback é uma classe do próprio DEX que guarda o
//! que recebe. Assim o teste passa pelo dispatch real, não só por
//! `androidx::call_*`.

mod common;

use common::*;
use rd_vm::engine::VmConfig;
use rd_vm::err::VmExit;
use rd_vm::heap::ObjRef;
use rd_vm::value::Value;
use rd_vm::Engine;

const CB: &str = "Lcom/example/Cb;";
const APP: &str = "LCaso;";
const MGR: &str = "Landroidx/credentials/CredentialManager;";
const CTX: &str = "Landroid/content/Context;";
const NPE: &str = "Ljava/lang/NullPointerException;";
const NO_CRED: &str = "Landroidx/credentials/exceptions/NoCredentialException;";
const CREATE_UNSUP: &str = "Landroidx/credentials/exceptions/CreateCredentialUnsupportedException;";
const TYPE_NO_CRED: &str = "android.credentials.GetCredentialException.TYPE_NO_CREDENTIAL";
const TYPE_CREATE_UNSUP: &str = "androidx.credentials.TYPE_CREATE_CREDENTIAL_UNSUPPORTED_EXCEPTION";

const MAKE_SIG: &str = "(Landroid/content/Context;)Landroidx/credentials/CredentialManager;";
const RUN_GET_SIG: &str = "(Landroidx/credentials/CredentialManager;Landroid/content/Context;Landroidx/credentials/GetCredentialRequest;Ljava/lang/Object;)V";
const RUN_CREATE_SIG: &str = "(Landroidx/credentials/CredentialManager;Landroid/content/Context;Landroidx/credentials/CreateCredentialRequest;Ljava/lang/Object;)V";
const RUN_CLEAR_SIG: &str = "(Landroidx/credentials/CredentialManager;Landroidx/credentials/ClearCredentialStateRequest;Ljava/lang/Object;)V";

const SENTINEL: i32 = 7;

/// Monta o DEX: callback que guarda `onError`/`onResult` em campos, e
/// métodos estáticos que fazem a chamada assíncrona com `null` em
/// CancellationSignal e Executor (opcionais).
fn build_dex() -> Vec<u8> {
    let mut b = DexBuilder::new();

    // Callback: this=v1, arg=v2 (regs 3, ins 2). iput-object arg → campo.
    let cb = b.class(CB, "Ljava/lang/Object;");
    b.instance_field(cb, "err", "Ljava/lang/Object;");
    b.instance_field(cb, "ok", "Ljava/lang/Object;");
    let f_err = b.field_idx(CB, "Ljava/lang/Object;", "err");
    let f_ok = b.field_idx(CB, "Ljava/lang/Object;", "ok");
    let on_error = [op22c(0x5B, 2, 1, f_err), op10x(0x0E)].concat();
    let on_result = [op22c(0x5B, 2, 1, f_ok), op10x(0x0E)].concat();
    b.r#virtual(
        cb,
        "onError",
        "V",
        vec!["Ljava/lang/Object;"],
        ACC_PUBLIC,
        Some(b.code(3, 2, 0, on_error)),
    );
    b.r#virtual(
        cb,
        "onResult",
        "V",
        vec!["Ljava/lang/Object;"],
        ACC_PUBLIC,
        Some(b.code(3, 2, 0, on_result)),
    );

    let app = b.class(APP, "Ljava/lang/Object;");

    // make(Context): CredentialManager.create(ctx) → devolve o objeto.
    let p_create = b.proto_idx(MGR, vec![CTX.into()]);
    let m_create = b.method_idx(MGR, p_create, "create");
    let make = [
        op35c(0x71, 1, m_create, [0, 0, 0, 0, 0]), // invoke-static {v0}, create
        op11x(0x0C, 0),                            // move-result-object v0
        op11x(0x11, 0),                            // return-object v0
    ]
    .concat();
    b.direct(
        app,
        "make",
        MGR,
        vec![CTX],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 1, 1, make)),
    );

    // run*(mgr, …, cb): regs 6 (get/create) ou 5 (clear). Os argumentos da
    // interface vão em v0..vN contíguos para `invoke-interface/range`.
    let get_proto = b.proto_idx(
        "V",
        vec![
            CTX.into(),
            "Landroidx/credentials/GetCredentialRequest;".into(),
            "Landroid/os/CancellationSignal;".into(),
            "Ljava/util/concurrent/Executor;".into(),
            "Landroidx/credentials/CredentialManagerCallback;".into(),
        ],
    );
    let create_proto = b.proto_idx(
        "V",
        vec![
            CTX.into(),
            "Landroidx/credentials/CreateCredentialRequest;".into(),
            "Landroid/os/CancellationSignal;".into(),
            "Ljava/util/concurrent/Executor;".into(),
            "Landroidx/credentials/CredentialManagerCallback;".into(),
        ],
    );
    let clear_proto = b.proto_idx(
        "V",
        vec![
            "Landroidx/credentials/ClearCredentialStateRequest;".into(),
            "Landroid/os/CancellationSignal;".into(),
            "Ljava/util/concurrent/Executor;".into(),
            "Landroidx/credentials/CredentialManagerCallback;".into(),
        ],
    );
    let m_get = b.method_idx(MGR, get_proto, "getCredentialAsync");
    let m_create_async = b.method_idx(MGR, create_proto, "createCredentialAsync");
    let m_clear = b.method_idx(MGR, clear_proto, "clearCredentialStateAsync");

    // Get/create: params ins=4 (mgr, ctx, req, cb) em v2..v5, regs=6.
    // Move para v0..v5 = [mgr, ctx, req, null, null, cb].
    let forward_two = |method: u16, count_regs: u16| -> Vec<u16> {
        let mut u: Vec<u16> = [
            op12x(0x07, 0, 2), // move-object v0, v2 (mgr)
            op12x(0x07, 1, 3), // move-object v1, v3 (ctx)
            op12x(0x07, 2, 4), // move-object v2, v4 (req)
            op11n(0x12, 3, 0), // const/4 v3, 0 (cancel = null)
            op11n(0x12, 4, 0), // const/4 v4, 0 (executor = null)
        ]
        .concat();
        // cb continua em v5
        u.extend([((count_regs) << 8) | 0x78, method, 0]); // invoke-interface/range {v0..}
        u.extend(op10x(0x0E)); // return-void
        u
    };
    let run_get = forward_two(m_get, 6);
    let run_create = forward_two(m_create_async, 6);
    b.direct(
        app,
        "runGet",
        "V",
        vec![
            MGR,
            CTX,
            "Landroidx/credentials/GetCredentialRequest;",
            "Ljava/lang/Object;",
        ],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(6, 4, 0, run_get)),
    );
    b.direct(
        app,
        "runCreate",
        "V",
        vec![
            MGR,
            CTX,
            "Landroidx/credentials/CreateCredentialRequest;",
            "Ljava/lang/Object;",
        ],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(6, 4, 0, run_create)),
    );

    // Clear: params ins=3 (mgr, req, cb) em v2..v4, regs=5.
    // Move para v0..v4 = [mgr, req, null, null, cb].
    let run_clear = [
        op12x(0x07, 0, 2), // move-object v0, v2 (mgr)
        op12x(0x07, 1, 3), // move-object v1, v3 (req)
        op11n(0x12, 2, 0), // const/4 v2, 0 (cancel = null)
        op11n(0x12, 3, 0), // const/4 v3, 0 (executor = null)
        vec![(5u16 << 8) | 0x78, m_clear, 0],
        op10x(0x0E),
    ]
    .concat();
    b.direct(
        app,
        "runClear",
        "V",
        vec![
            MGR,
            "Landroidx/credentials/ClearCredentialStateRequest;",
            "Ljava/lang/Object;",
        ],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(5, 3, 0, run_clear)),
    );

    b.finish()
}

fn engine() -> Engine {
    let dex = rd_dex::Dex::parse(build_dex()).expect("DEX sintético parseia");
    Engine::new(vec![dex], VmConfig::default())
}

fn alloc(vm: &mut Engine, class: &str) -> Value {
    Value::Obj(
        vm.heap
            .alloc_instance(class.to_string(), Vec::new())
            .unwrap(),
    )
}

/// Callback com campos iniciais `SENTINEL`, para distinguir "não chamado"
/// de "chamado com null".
fn callback(vm: &mut Engine) -> ObjRef {
    vm.heap
        .alloc_instance(
            CB.to_string(),
            vec![
                ("err".to_string(), Value::Int(SENTINEL)),
                ("ok".to_string(), Value::Int(SENTINEL)),
            ],
        )
        .unwrap()
}

fn field(vm: &Engine, r: ObjRef, name: &str) -> Value {
    vm.heap.get_field(r, name).unwrap()
}

fn text(vm: &Engine, v: &Value) -> String {
    match v {
        Value::Obj(r) => vm.heap.as_str(*r).unwrap().to_string(),
        other => panic!("esperava String, veio {other:?}"),
    }
}

fn manager(vm: &mut Engine) -> Value {
    let ctx = alloc(vm, CTX);
    invoke(vm, "make", MAKE_SIG, &[ctx]).expect("CredentialManager.create")
}

#[test]
fn create_returns_host_manager_through_static_dispatch() {
    let mut vm = engine();
    let mgr = manager(&mut vm);
    let Value::Obj(r) = mgr else {
        panic!("esperava objeto");
    };
    assert_eq!(
        vm.heap.class_of(r).unwrap(),
        "Landroidx/credentials/CredentialManagerImpl;"
    );
}

#[test]
fn get_without_provider_delivers_no_credential_exception_with_library_type() {
    let mut vm = engine();
    let mgr = manager(&mut vm);
    let ctx = alloc(&mut vm, CTX);
    let req = alloc(&mut vm, "Landroidx/credentials/GetCredentialRequest;");
    let cb = callback(&mut vm);
    invoke(
        &mut vm,
        "runGet",
        RUN_GET_SIG,
        &[mgr, ctx, req, Value::Obj(cb)],
    )
    .expect("getCredentialAsync");

    // onError recebeu a exceção; onResult nunca foi chamado.
    let Value::Obj(exc) = field(&vm, cb, "err") else {
        panic!("onError não foi chamado");
    };
    assert_eq!(vm.heap.class_of(exc).unwrap(), NO_CRED);
    assert_eq!(text(&vm, &field(&vm, exc, "type")), TYPE_NO_CRED);
    assert!(text(&vm, &field(&vm, exc, "errorMessage")).contains("nenhum provedor"));
    assert_eq!(field(&vm, cb, "ok"), Value::Int(SENTINEL));
}

#[test]
fn create_without_provider_delivers_unsupported_exception() {
    let mut vm = engine();
    let mgr = manager(&mut vm);
    let ctx = alloc(&mut vm, CTX);
    let req = alloc(&mut vm, "Landroidx/credentials/CreateCredentialRequest;");
    let cb = callback(&mut vm);
    invoke(
        &mut vm,
        "runCreate",
        RUN_CREATE_SIG,
        &[mgr, ctx, req, Value::Obj(cb)],
    )
    .expect("createCredentialAsync");

    let Value::Obj(exc) = field(&vm, cb, "err") else {
        panic!("onError não foi chamado");
    };
    assert_eq!(vm.heap.class_of(exc).unwrap(), CREATE_UNSUP);
    assert_eq!(text(&vm, &field(&vm, exc, "type")), TYPE_CREATE_UNSUP);
}

#[test]
fn clear_state_delivers_null_result() {
    let mut vm = engine();
    let mgr = manager(&mut vm);
    let req = alloc(
        &mut vm,
        "Landroidx/credentials/ClearCredentialStateRequest;",
    );
    let cb = callback(&mut vm);
    invoke(
        &mut vm,
        "runClear",
        RUN_CLEAR_SIG,
        &[mgr, req, Value::Obj(cb)],
    )
    .expect("clearCredentialStateAsync");

    assert_eq!(field(&vm, cb, "ok"), Value::Null);
    assert_eq!(field(&vm, cb, "err"), Value::Int(SENTINEL));
}

#[test]
fn null_callback_is_npe_and_nothing_is_delivered() {
    let mut vm = engine();
    let mgr = manager(&mut vm);
    let ctx = alloc(&mut vm, CTX);
    let req = alloc(&mut vm, "Landroidx/credentials/GetCredentialRequest;");
    let err = invoke(
        &mut vm,
        "runGet",
        RUN_GET_SIG,
        &[mgr, ctx, req, Value::Null],
    )
    .expect_err("callback nulo deve falhar");
    match err {
        VmExit::Exception(t) => assert_eq!(t.class, NPE),
        other => panic!("esperava NPE, veio {other:?}"),
    }
}

#[test]
fn null_context_is_npe_before_any_callback() {
    let mut vm = engine();
    let mgr = manager(&mut vm);
    let req = alloc(&mut vm, "Landroidx/credentials/GetCredentialRequest;");
    let cb = callback(&mut vm);
    let err = invoke(
        &mut vm,
        "runGet",
        RUN_GET_SIG,
        &[mgr, Value::Null, req, Value::Obj(cb)],
    )
    .expect_err("contexto nulo deve falhar");
    match err {
        VmExit::Exception(t) => assert_eq!(t.class, NPE),
        other => panic!("esperava NPE, veio {other:?}"),
    }
    assert_eq!(field(&vm, cb, "err"), Value::Int(SENTINEL));
}
