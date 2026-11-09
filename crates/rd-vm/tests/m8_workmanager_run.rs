//! WorkManager: `Worker.doWork()` executado pelo harness (`Engine::m8_run_work`).
//! Os workers são classes DEX sintéticas que estendem `androidx.work.Worker`
//! (superclasse host). Cobre o mapeamento de `ListenableWorker.Result`
//! (success / failure / retry / null), progresso (`setProgressAsync`) e output,
//! lidos pelos getters de `WorkInfo` e preservados pelo GC.

use rd_testsupport::vm::*;
use rd_vm::m8::*;
use rd_vm::value::Value;
use rd_vm::Engine;

const BUILDER: &str = "Landroidx/work/OneTimeWorkRequest$Builder;";
const REQUEST: &str = "Landroidx/work/OneTimeWorkRequest;";
const WM: &str = "Landroidx/work/WorkManager;";
const RESULT: &str = "Landroidx/work/ListenableWorker$Result;";
const PARAMS: &str = "Landroidx/work/WorkerParameters;";
const DATA: &str = "Landroidx/work/Data;";
const DATA_BUILDER: &str = "Landroidx/work/Data$Builder;";
const FUTURE: &str = "Lcom/google/common/util/concurrent/ListenableFuture;";
const PROBE: &str = "LWmRunProbe;";
/// Construtor: `ACC_PUBLIC` com a flag de construtor (0x10000).
const ACC_CTOR: u32 = ACC_PUBLIC | 0x1_0000;

#[derive(Clone, Copy)]
enum Body {
    Success,
    Failure,
    Retry,
    Null,
    Progress,
}

/// Worker sintético: `extends Worker`, com `<init>(Context, WorkerParameters)`
/// que chama o `super` e um `doWork()` com o corpo pedido.
fn add_worker(b: &mut DexBuilder, class_desc: &str, body: Body) {
    let cls = b.class(class_desc, WORKER_BASE);

    // <init>(ctx, params): super.<init>(ctx, params); return-void
    // ins=3 → this=v0, ctx=v1, params=v2.
    let p_super = b.proto_idx("V", vec!["Landroid/content/Context;".into(), PARAMS.into()]);
    let m_super = b.method_idx(WORKER_BASE, p_super, "<init>");
    let mut u = Vec::new();
    u.extend(op35c(0x70, 3, m_super, [0, 1, 2, 0, 0])); // invoke-direct
    u.extend(op10x(0x0e)); // return-void
    let code = b.code(3, 3, 3, u);
    b.direct(
        cls,
        "<init>",
        "V",
        vec!["Landroid/content/Context;", PARAMS],
        ACC_CTOR,
        Some(code),
    );

    // doWork(): ins=1 → this=v4 (regs=5).
    let mut u = Vec::new();
    match body {
        Body::Success | Body::Failure | Body::Retry => {
            let name = match body {
                Body::Success => "success",
                Body::Failure => "failure",
                _ => "retry",
            };
            let p = b.proto_idx(RESULT, vec![]);
            let m = b.method_idx(RESULT, p, name);
            u.extend(op35c(0x71, 0, m, [0, 0, 0, 0, 0])); // invoke-static {}
            u.extend(op11x(0x0c, 0)); // move-result-object v0
            u.extend(op11x(0x11, 0)); // return-object v0
        }
        Body::Null => {
            u.extend(op11n(0x12, 0, 0)); // const/4 v0, 0
            u.extend(op11x(0x11, 0)); // return-object v0
        }
        Body::Progress => {
            // Data d = new Data.Builder().putInt("pct", 50).build();
            // setProgressAsync(d); return Result.success(d);
            let t_db = b.type_idx(DATA_BUILDER);
            let p_binit = b.proto_idx("V", vec![]);
            let m_binit = b.method_idx(DATA_BUILDER, p_binit, "<init>");
            let s_pct = b.intern("pct");
            let p_put = b.proto_idx(DATA_BUILDER, vec!["Ljava/lang/String;".into(), "I".into()]);
            let m_put = b.method_idx(DATA_BUILDER, p_put, "putInt");
            let p_build = b.proto_idx(DATA, vec![]);
            let m_build = b.method_idx(DATA_BUILDER, p_build, "build");
            let p_prog = b.proto_idx(FUTURE, vec![DATA.into()]);
            let m_prog = b.method_idx(WORKER_BASE, p_prog, "setProgressAsync");
            let p_succ = b.proto_idx(RESULT, vec![DATA.into()]);
            let m_succ = b.method_idx(RESULT, p_succ, "success");
            u.extend(op21c(0x22, 0, t_db)); // new-instance v0, Data$Builder
            u.extend(op35c(0x70, 1, m_binit, [0, 0, 0, 0, 0])); // invoke-direct {v0} <init>
            u.extend(op21c(0x1a, 1, s_pct as u16)); // const-string v1, "pct"
            u.extend(op21s(0x13, 2, 50)); // const/16 v2, 50
            u.extend(op35c(0x6e, 3, m_put, [0, 1, 2, 0, 0])); // invoke-virtual putInt
            u.extend(op35c(0x6e, 1, m_build, [0, 0, 0, 0, 0])); // invoke-virtual build
            u.extend(op11x(0x0c, 3)); // move-result-object v3
            u.extend(op35c(0x6e, 2, m_prog, [4, 3, 0, 0, 0])); // this.setProgressAsync(v3)
            u.extend(op35c(0x71, 1, m_succ, [3, 0, 0, 0, 0])); // Result.success(v3)
            u.extend(op11x(0x0c, 0)); // move-result-object v0
            u.extend(op11x(0x11, 0)); // return-object v0
        }
    }
    let code = b.code(5, 1, 2, u);
    b.r#virtual(cls, "doWork", RESULT, vec![], ACC_PUBLIC, Some(code));
}

/// `Builder(Worker.class).addTag("sync").build()` e `WorkManager.enqueue`,
/// num método estático `name(Context)`.
fn add_enqueue(b: &mut DexBuilder, probe: usize, name: &str, worker_desc: &str) {
    let t_builder = b.type_idx(BUILDER);
    let t_worker = b.type_idx(worker_desc);
    let s_tag = b.intern("sync");
    let p_init = b.proto_idx("V", vec!["Ljava/lang/Class;".into()]);
    let m_init = b.method_idx(BUILDER, p_init, "<init>");
    let p_tag = b.proto_idx(
        "Landroidx/work/WorkRequest$Builder;",
        vec!["Ljava/lang/String;".into()],
    );
    let m_tag = b.method_idx(BUILDER, p_tag, "addTag");
    let p_build = b.proto_idx(REQUEST, vec![]);
    let m_build = b.method_idx(BUILDER, p_build, "build");
    let p_get = b.proto_idx(WM, vec!["Landroid/content/Context;".into()]);
    let m_get = b.method_idx(WM, p_get, "getInstance");
    let p_enq = b.proto_idx(
        "Landroidx/work/Operation;",
        vec!["Landroidx/work/WorkRequest;".into()],
    );
    let m_enq = b.method_idx(WM, p_enq, "enqueue");

    // v0 builder, v1 Class, v2 tag, v3 pedido, v4 WorkManager, v5 Context.
    let mut u = Vec::new();
    u.extend(op21c(0x22, 0, t_builder)); // new-instance v0, Builder
    u.extend(op21c(0x1c, 1, t_worker)); // const-class v1, worker
    u.extend(op35c(0x70, 2, m_init, [0, 1, 0, 0, 0])); // invoke-direct <init>(Class)
    u.extend(op21c(0x1a, 2, s_tag as u16)); // const-string v2, "sync"
    u.extend(op35c(0x6e, 2, m_tag, [0, 2, 0, 0, 0])); // addTag
    u.extend(op35c(0x6e, 1, m_build, [0, 0, 0, 0, 0])); // build
    u.extend(op11x(0x0c, 3)); // move-result-object v3
    u.extend(op35c(0x71, 1, m_get, [5, 0, 0, 0, 0])); // getInstance(ctx)
    u.extend(op11x(0x0c, 4)); // move-result-object v4
    u.extend(op35c(0x6e, 2, m_enq, [4, 3, 0, 0, 0])); // enqueue
    u.extend(op10x(0x0e)); // return-void
    let code = b.code(6, 1, 3, u);
    b.direct(
        probe,
        name,
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(code),
    );
}

/// Engine com os workers sintéticos e um probe por tipo de worker.
fn engine_with_workers() -> Engine {
    let mut b = DexBuilder::new();
    let probe = b.class(PROBE, "Ljava/lang/Object;");
    add_worker(&mut b, "Lcom/example/RunWorker;", Body::Success);
    add_worker(&mut b, "Lcom/example/FailWorker;", Body::Failure);
    add_worker(&mut b, "Lcom/example/RetryWorker;", Body::Retry);
    add_worker(&mut b, "Lcom/example/NullWorker;", Body::Null);
    add_worker(&mut b, "Lcom/example/ProgressWorker;", Body::Progress);
    add_enqueue(&mut b, probe, "enqueueRun", "Lcom/example/RunWorker;");
    add_enqueue(&mut b, probe, "enqueueFail", "Lcom/example/FailWorker;");
    add_enqueue(&mut b, probe, "enqueueRetry", "Lcom/example/RetryWorker;");
    add_enqueue(&mut b, probe, "enqueueNull", "Lcom/example/NullWorker;");
    add_enqueue(
        &mut b,
        probe,
        "enqueueProgress",
        "Lcom/example/ProgressWorker;",
    );
    engine_of(&b)
}

/// Enfileira pelo probe DEX e devolve o UUID do pedido criado.
fn enqueue(e: &mut Engine, method: &str) -> String {
    e.invoke_static(
        PROBE,
        method,
        "(Landroid/content/Context;)V",
        &[Value::Null],
    )
    .expect("enqueue DEX");
    e.m8_work().last().expect("pedido").id.clone()
}

fn state_of(e: &Engine, id: &str) -> String {
    e.m8_work()
        .iter()
        .find(|w| w.id == id)
        .expect("pedido")
        .state
        .clone()
}

/// `WorkInfo` vivo do pedido com a tag "sync", pelo caminho de despacho real
/// (`getWorkInfosByTag` → future → lista host).
fn work_info_tagged_sync(e: &mut Engine) -> u32 {
    let wm = match call_m8_static(
        e,
        WORK_MANAGER,
        "getInstance",
        "(Landroid/content/Context;)Landroidx/work/WorkManager;",
        &[Value::Null],
    )
    .unwrap()
    {
        Some(Value::Obj(w)) => w,
        other => panic!("WorkManager: {other:?}"),
    };
    let tag = Value::Obj(e.heap.alloc_string("sync".to_string()).unwrap());
    let fut = match call_m8_instance(
        e,
        WORK_MANAGER,
        "getWorkInfosByTag",
        "(Ljava/lang/String;)Lcom/google/common/util/concurrent/ListenableFuture;",
        wm,
        &[tag],
    )
    .unwrap()
    {
        Some(Value::Obj(f)) => f,
        other => panic!("future: {other:?}"),
    };
    let list = match e.fw.androidx.futures.get(&fut).map(|f| f.result.clone()) {
        Some(Value::Obj(l)) => l,
        other => panic!("future sem lista: {other:?}"),
    };
    match e.fw.objects.get(&list) {
        Some(rd_vm::framework::HostObj::List { elems }) => match elems.first() {
            Some(Value::Obj(i)) => *i,
            other => panic!("WorkInfo: {other:?}"),
        },
        other => panic!("lista host: {other:?}"),
    }
}

/// Entradas de um `Data` host, em ordem.
fn data_entries(e: &Engine, d: u32) -> Vec<(String, Value)> {
    e.androidx_work_data(d).expect("Data host")
}

#[test]
fn success_runs_do_work_and_marks_succeeded() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueRun");
    assert_eq!(
        state_of(&e, &id),
        "ENQUEUED",
        "antes de rodar, fica enfileirado"
    );
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
    assert_eq!(state_of(&e, &id), "SUCCEEDED");
    let rec = e.m8_work().iter().find(|w| w.id == id).unwrap();
    assert!(rec.output.is_none(), "success() sem Data não tem output");
}

#[test]
fn failure_marks_failed() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueFail");
    assert_eq!(e.m8_run_work(&id).unwrap(), "FAILED");
    assert_eq!(state_of(&e, &id), "FAILED");
}

#[test]
fn retry_goes_back_to_enqueued_and_can_run_again() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueRetry");
    assert_eq!(e.m8_run_work(&id).unwrap(), "ENQUEUED", "retry() reagenda");
    assert_eq!(state_of(&e, &id), "ENQUEUED");
    // Reagendado continua executável.
    assert_eq!(e.m8_run_work(&id).unwrap(), "ENQUEUED");
}

#[test]
fn null_result_is_failure() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueNull");
    assert_eq!(
        e.m8_run_work(&id).unwrap(),
        "FAILED",
        "doWork() null = falha"
    );
}

#[test]
fn run_only_touches_enqueued_work() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueRun");
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
    // Já concluído: devolve o estado, sem rodar de novo.
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
    assert_eq!(
        e.m8_run_work("00000000-0000-0000-0000-000000000000")
            .unwrap(),
        ""
    );
}

#[test]
fn progress_and_output_come_from_do_work() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueProgress");
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
    let rec = e.m8_work().iter().find(|w| w.id == id).unwrap().clone();
    let progress = rec.progress.expect("setProgressAsync gravou o progresso");
    assert_eq!(
        data_entries(&e, progress),
        vec![("pct".to_string(), Value::Int(50))]
    );
    let output = rec.output.expect("Result.success(Data) gravou o output");
    assert_eq!(
        data_entries(&e, output),
        vec![("pct".to_string(), Value::Int(50))]
    );
}

#[test]
fn work_info_getters_expose_progress_and_output() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueProgress");
    e.m8_run_work(&id).unwrap();
    let info = work_info_tagged_sync(&mut e);
    let progress = match call_m8_instance(
        &mut e,
        "Landroidx/work/WorkInfo;",
        "getProgress",
        "()Landroidx/work/Data;",
        info,
        &[],
    )
    .unwrap()
    {
        Some(Value::Obj(d)) => d,
        other => panic!("getProgress: {other:?}"),
    };
    assert_eq!(
        data_entries(&e, progress),
        vec![("pct".to_string(), Value::Int(50))]
    );
    let output = match call_m8_instance(
        &mut e,
        "Landroidx/work/WorkInfo;",
        "getOutputData",
        "()Landroidx/work/Data;",
        info,
        &[],
    )
    .unwrap()
    {
        Some(Value::Obj(d)) => d,
        other => panic!("getOutputData: {other:?}"),
    };
    assert_eq!(
        data_entries(&e, output),
        vec![("pct".to_string(), Value::Int(50))]
    );
}

#[test]
fn work_info_without_run_has_empty_progress_and_output() {
    let mut e = engine_with_workers();
    enqueue(&mut e, "enqueueRun");
    let info = work_info_tagged_sync(&mut e);
    for m in ["getProgress", "getOutputData"] {
        let d = match call_m8_instance(
            &mut e,
            "Landroidx/work/WorkInfo;",
            m,
            "()Landroidx/work/Data;",
            info,
            &[],
        )
        .unwrap()
        {
            Some(Value::Obj(d)) => d,
            other => panic!("{m}: {other:?}"),
        };
        assert!(data_entries(&e, d).is_empty(), "{m} vazio antes de rodar");
    }
}

#[test]
fn progress_and_output_survive_garbage_collection() {
    let mut e = engine_with_workers();
    let id = enqueue(&mut e, "enqueueProgress");
    e.m8_run_work(&id).unwrap();
    let _ = e.gc();
    let rec = e.m8_work().iter().find(|w| w.id == id).unwrap().clone();
    let progress = rec.progress.expect("progresso");
    assert_eq!(
        data_entries(&e, progress),
        vec![("pct".to_string(), Value::Int(50))],
        "raiz do GC mantém o Data de progresso"
    );
}

// ── Constraints (S24) ───────────────────────────────────────────────────

/// `Constraints.Builder()` com as constraints pedidas, `WorkRequest.Builder(worker)
/// .setConstraints(c).addTag("sync").build()` e `enqueue`, num método `name(Context)`.
fn add_constrained_enqueue(
    b: &mut DexBuilder,
    probe: usize,
    name: &str,
    worker_desc: &str,
    net: Option<&str>,
    charging: bool,
    idle: bool,
) {
    let t_cb = b.type_idx(CONSTRAINTS_BUILDER);
    let p_cb_init = b.proto_idx("V", vec![]);
    let m_cb_init = b.method_idx(CONSTRAINTS_BUILDER, p_cb_init, "<init>");
    let p_net = b.proto_idx(CONSTRAINTS_BUILDER, vec![NETWORK_TYPE.into()]);
    let m_net = b.method_idx(CONSTRAINTS_BUILDER, p_net, "setRequiredNetworkType");
    let p_bool = b.proto_idx(CONSTRAINTS_BUILDER, vec!["Z".into()]);
    let m_chg = b.method_idx(CONSTRAINTS_BUILDER, p_bool, "setRequiresCharging");
    let m_idle = b.method_idx(CONSTRAINTS_BUILDER, p_bool, "setRequiresDeviceIdle");
    let p_cbuild = b.proto_idx(CONSTRAINTS, vec![]);
    let m_cbuild = b.method_idx(CONSTRAINTS_BUILDER, p_cbuild, "build");
    let p_setc = b.proto_idx(
        "Landroidx/work/WorkRequest$Builder;",
        vec![CONSTRAINTS.into()],
    );
    let m_setc = b.method_idx(BUILDER, p_setc, "setConstraints");

    let t_builder = b.type_idx(BUILDER);
    let t_worker = b.type_idx(worker_desc);
    let s_tag = b.intern("sync");
    let p_init = b.proto_idx("V", vec!["Ljava/lang/Class;".into()]);
    let m_init = b.method_idx(BUILDER, p_init, "<init>");
    let p_tag = b.proto_idx(
        "Landroidx/work/WorkRequest$Builder;",
        vec!["Ljava/lang/String;".into()],
    );
    let m_tag = b.method_idx(BUILDER, p_tag, "addTag");
    let p_build = b.proto_idx(REQUEST, vec![]);
    let m_wbuild = b.method_idx(BUILDER, p_build, "build");
    let p_get = b.proto_idx(WM, vec!["Landroid/content/Context;".into()]);
    let m_get = b.method_idx(WM, p_get, "getInstance");
    let p_enq = b.proto_idx(
        "Landroidx/work/Operation;",
        vec!["Landroidx/work/WorkRequest;".into()],
    );
    let m_enq = b.method_idx(WM, p_enq, "enqueue");

    // regs=10, ins=1 → Context em v9.
    let mut u = Vec::new();
    u.extend(op21c(0x22, 0, t_cb)); // new-instance v0, Constraints.Builder
    u.extend(op35c(0x70, 1, m_cb_init, [0, 0, 0, 0, 0])); // <init>()
    if let Some(net) = net {
        let f = b.field_idx(NETWORK_TYPE, NETWORK_TYPE, net);
        u.extend(op21c(0x62, 1, f)); // sget-object v1, NetworkType.<net>
        u.extend(op35c(0x6e, 2, m_net, [0, 1, 0, 0, 0])); // setRequiredNetworkType
    }
    if charging {
        u.extend(op11n(0x12, 1, 1)); // const/4 v1, 1
        u.extend(op35c(0x6e, 2, m_chg, [0, 1, 0, 0, 0])); // setRequiresCharging(true)
    }
    if idle {
        u.extend(op11n(0x12, 1, 1));
        u.extend(op35c(0x6e, 2, m_idle, [0, 1, 0, 0, 0])); // setRequiresDeviceIdle(true)
    }
    u.extend(op35c(0x6e, 1, m_cbuild, [0, 0, 0, 0, 0])); // build()
    u.extend(op11x(0x0c, 2)); // move-result-object v2 (Constraints)
    u.extend(op21c(0x22, 3, t_builder)); // new-instance v3, OneTimeWorkRequest.Builder
    u.extend(op21c(0x1c, 4, t_worker)); // const-class v4, worker
    u.extend(op35c(0x70, 2, m_init, [3, 4, 0, 0, 0])); // <init>(Class)
    u.extend(op35c(0x6e, 2, m_setc, [3, 2, 0, 0, 0])); // setConstraints(v2)
    u.extend(op21c(0x1a, 5, s_tag as u16)); // const-string v5, "sync"
    u.extend(op35c(0x6e, 2, m_tag, [3, 5, 0, 0, 0])); // addTag
    u.extend(op35c(0x6e, 1, m_wbuild, [3, 0, 0, 0, 0])); // build()
    u.extend(op11x(0x0c, 6)); // move-result-object v6 (pedido)
    u.extend(op35c(0x71, 1, m_get, [9, 0, 0, 0, 0])); // getInstance(ctx)
    u.extend(op11x(0x0c, 7)); // move-result-object v7
    u.extend(op35c(0x6e, 2, m_enq, [7, 6, 0, 0, 0])); // enqueue
    u.extend(op10x(0x0e)); // return-void
    let code = b.code(10, 1, 3, u);
    b.direct(
        probe,
        name,
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(code),
    );
}

/// Engine com RunWorker/ProgressWorker e probes com constraints.
fn engine_with_constraints() -> Engine {
    let mut b = DexBuilder::new();
    let probe = b.class(PROBE, "Ljava/lang/Object;");
    add_worker(&mut b, "Lcom/example/RunWorker;", Body::Success);
    add_worker(&mut b, "Lcom/example/ProgressWorker;", Body::Progress);
    add_constrained_enqueue(
        &mut b,
        probe,
        "enqueueNone",
        "Lcom/example/RunWorker;",
        None,
        false,
        false,
    );
    add_constrained_enqueue(
        &mut b,
        probe,
        "enqueueCharging",
        "Lcom/example/ProgressWorker;",
        None,
        true,
        false,
    );
    add_constrained_enqueue(
        &mut b,
        probe,
        "enqueueConnected",
        "Lcom/example/RunWorker;",
        Some("CONNECTED"),
        false,
        false,
    );
    add_constrained_enqueue(
        &mut b,
        probe,
        "enqueueUnmetered",
        "Lcom/example/RunWorker;",
        Some("UNMETERED"),
        false,
        false,
    );
    add_constrained_enqueue(
        &mut b,
        probe,
        "enqueueIdle",
        "Lcom/example/RunWorker;",
        None,
        false,
        true,
    );
    engine_of(&b)
}

#[test]
fn constraints_reach_the_work_record() {
    let mut e = engine_with_constraints();
    let id = enqueue(&mut e, "enqueueCharging");
    let rec = e.m8_work().iter().find(|w| w.id == id).unwrap().clone();
    assert_eq!(
        rec.constraints,
        WorkConstraints {
            network: None,
            charging: true,
            ..Default::default()
        },
        "setConstraints/build levam as constraints ao pedido"
    );
}

#[test]
fn without_constraints_runs_even_offline() {
    let mut e = engine_with_constraints();
    e.m8_set_device(DeviceConditions {
        network: NetworkCondition::None,
        ..Default::default()
    });
    let id = enqueue(&mut e, "enqueueNone");
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
}

#[test]
fn charging_constraint_blocks_until_plugged_in() {
    let mut e = engine_with_constraints();
    let id = enqueue(&mut e, "enqueueCharging");
    // Não está na tomada: fica ENQUEUED e o doWork não roda (sem progresso).
    assert_eq!(e.m8_run_work(&id).unwrap(), "ENQUEUED");
    assert_eq!(state_of(&e, &id), "ENQUEUED");
    assert!(
        e.m8_work()
            .iter()
            .find(|w| w.id == id)
            .unwrap()
            .progress
            .is_none(),
        "doWork não roda com constraint não atendida"
    );
    e.m8_set_device(DeviceConditions {
        charging: true,
        ..Default::default()
    });
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
    assert!(e
        .m8_work()
        .iter()
        .find(|w| w.id == id)
        .unwrap()
        .progress
        .is_some());
}

#[test]
fn connected_constraint_blocks_when_offline() {
    let mut e = engine_with_constraints();
    e.m8_set_device(DeviceConditions {
        network: NetworkCondition::None,
        ..Default::default()
    });
    let id = enqueue(&mut e, "enqueueConnected");
    assert_eq!(e.m8_run_work(&id).unwrap(), "ENQUEUED");
    e.m8_set_device(DeviceConditions {
        network: NetworkCondition::Metered,
        ..Default::default()
    });
    assert_eq!(
        e.m8_run_work(&id).unwrap(),
        "SUCCEEDED",
        "CONNECTED aceita rede medida"
    );
}

#[test]
fn unmetered_constraint_rejects_metered_network() {
    let mut e = engine_with_constraints();
    e.m8_set_device(DeviceConditions {
        network: NetworkCondition::Metered,
        ..Default::default()
    });
    let id = enqueue(&mut e, "enqueueUnmetered");
    assert_eq!(
        e.m8_run_work(&id).unwrap(),
        "ENQUEUED",
        "UNMETERED recusa rede medida"
    );
    e.m8_set_device(DeviceConditions {
        network: NetworkCondition::Unmetered,
        ..Default::default()
    });
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
}

#[test]
fn idle_constraint_waits_for_idle_device() {
    let mut e = engine_with_constraints();
    let id = enqueue(&mut e, "enqueueIdle");
    assert_eq!(e.m8_run_work(&id).unwrap(), "ENQUEUED");
    e.m8_set_device(DeviceConditions {
        idle: true,
        ..Default::default()
    });
    assert_eq!(e.m8_run_work(&id).unwrap(), "SUCCEEDED");
}
