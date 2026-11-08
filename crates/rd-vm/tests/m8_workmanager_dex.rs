//! WorkManager (M8) exercitado por bytecode DEX sintético em memória: o app
//! monta o pedido com o builder, lê o `getId()`, enfileira e cancela por id.
//! Os efeitos são conferidos no registro de `Engine::m8_work()`.

use rd_testsupport::vm::*;
use rd_vm::Value;

const BUILDER: &str = "Landroidx/work/OneTimeWorkRequest$Builder;";
const REQUEST: &str = "Landroidx/work/OneTimeWorkRequest;";
const WM: &str = "Landroidx/work/WorkManager;";
const WORKER: &str = "Lcom/example/SyncWorker;";

#[test]
fn dex_builder_tags_id_enqueue_and_cancel_by_id() {
    let mut b = DexBuilder::new();
    let probe = b.class("LWmProbe;", "Ljava/lang/Object;");

    let p_get = b.proto_idx(WM, vec!["Landroid/content/Context;".into()]);
    let m_get = b.method_idx(WM, p_get, "getInstance");
    let p_init = b.proto_idx("V", vec!["Ljava/lang/Class;".into()]);
    let m_init = b.method_idx(BUILDER, p_init, "<init>");
    let p_tag = b.proto_idx(
        "Landroidx/work/WorkRequest$Builder;",
        vec!["Ljava/lang/String;".into()],
    );
    let m_tag = b.method_idx(BUILDER, p_tag, "addTag");
    let p_build = b.proto_idx(REQUEST, vec![]);
    let m_build = b.method_idx(BUILDER, p_build, "build");
    let p_enq = b.proto_idx(
        "Landroidx/work/Operation;",
        vec!["Landroidx/work/WorkRequest;".into()],
    );
    let m_enq = b.method_idx(WM, p_enq, "enqueue");
    let p_id = b.proto_idx("Ljava/util/UUID;", vec![]);
    let m_id = b.method_idx(REQUEST, p_id, "getId");
    let p_cancel = b.proto_idx("Landroidx/work/Operation;", vec!["Ljava/util/UUID;".into()]);
    let m_cancel = b.method_idx(WM, p_cancel, "cancelWorkById");

    let t_builder = b.type_idx(BUILDER);
    let t_worker = b.type_idx(WORKER);
    let s_tag = b.intern("sync");

    // Registros: v0 builder, v1 Class, v2 tag/UUID, v3 pedido, v4 WorkManager,
    // v5 Context (único parâmetro, registro final: regs=6, ins=1).
    let mut u = Vec::new();
    u.extend(op21c(0x22, 0, t_builder)); // new-instance v0, Builder
    u.extend(op21c(0x1c, 1, t_worker)); // const-class v1, SyncWorker
    u.extend(op35c(0x70, 2, m_init, [0, 1, 0, 0, 0])); // invoke-direct {v0,v1} <init>(Class)
    u.extend(op21c(0x1a, 2, s_tag as u16)); // const-string v2, "sync"
    u.extend(op35c(0x6e, 2, m_tag, [0, 2, 0, 0, 0])); // invoke-virtual {v0,v2} addTag
    u.extend(op35c(0x6e, 1, m_build, [0, 0, 0, 0, 0])); // invoke-virtual {v0} build
    u.extend(op11x(0x0c, 3)); // move-result-object v3
    u.extend(op35c(0x71, 1, m_get, [5, 0, 0, 0, 0])); // invoke-static {v5} getInstance
    u.extend(op11x(0x0c, 4)); // move-result-object v4
    u.extend(op35c(0x6e, 2, m_enq, [4, 3, 0, 0, 0])); // invoke-virtual {v4,v3} enqueue
    u.extend(op35c(0x6e, 1, m_id, [3, 0, 0, 0, 0])); // invoke-virtual {v3} getId
    u.extend(op11x(0x0c, 2)); // move-result-object v2
    u.extend(op35c(0x6e, 2, m_cancel, [4, 2, 0, 0, 0])); // invoke-virtual {v4,v2} cancelWorkById
    u.extend(op10x(0x0e)); // return-void
    let code = b.code(6, 1, 3, u);
    b.direct(
        probe,
        "run",
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(code),
    );

    let mut e = engine_of(&b);
    e.invoke_static(
        "LWmProbe;",
        "run",
        "(Landroid/content/Context;)V",
        &[Value::Null],
    )
    .expect("run");

    let work = e.m8_work();
    assert_eq!(work.len(), 1, "um pedido enfileirado");
    assert_eq!(
        work[0].worker_class, WORKER,
        "worker do construtor do builder"
    );
    assert_eq!(work[0].tags, vec!["sync".to_string()], "tag do addTag");
    assert_eq!(
        work[0].state, "CANCELLED",
        "cancelWorkById pelo getId do pedido"
    );
    assert_eq!(work[0].id.len(), 36, "UUID canônico do pedido");
}

const WORK_INFO: &str = "Landroidx/work/WorkInfo;";
const LIST: &str = "Ljava/util/List;";
const FUTURE: &str = "Lcom/google/common/util/concurrent/ListenableFuture;";
const STATE: &str = "Landroidx/work/WorkInfo$State;";

/// Monta o `Builder(SyncWorker.class).addTag("sync").build()` e enfileira.
fn enqueue_tagged_code(b: &mut DexBuilder) -> Vec<u16> {
    let t_builder = b.type_idx(BUILDER);
    let t_worker = b.type_idx(WORKER);
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
    // v0 builder, v1 Class, v2 tag, v3 pedido, v4 WorkManager, v5 Context
    let mut u = Vec::new();
    u.extend(op21c(0x22, 0, t_builder));
    u.extend(op21c(0x1c, 1, t_worker));
    u.extend(op35c(0x70, 2, m_init, [0, 1, 0, 0, 0]));
    u.extend(op21c(0x1a, 2, s_tag as u16));
    u.extend(op35c(0x6e, 2, m_tag, [0, 2, 0, 0, 0]));
    u.extend(op35c(0x6e, 1, m_build, [0, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 3));
    u.extend(op35c(0x71, 1, m_get, [5, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 4));
    u.extend(op35c(0x6e, 2, m_enq, [4, 3, 0, 0, 0]));
    u.extend(op10x(0x0e));
    u
}

#[test]
fn dex_work_infos_expose_states_through_future_list_and_enum_identity() {
    let mut b = DexBuilder::new();
    let probe = b.class("LWiProbe;", "Ljava/lang/Object;");
    let p_get = b.proto_idx(WM, vec!["Landroid/content/Context;".into()]);
    let m_get = b.method_idx(WM, p_get, "getInstance");
    let p_by_tag = b.proto_idx(FUTURE, vec!["Ljava/lang/String;".into()]);
    let m_by_tag = b.method_idx(WM, p_by_tag, "getWorkInfosByTag");
    let p_fget = b.proto_idx("Ljava/lang/Object;", vec![]);
    let m_fget = b.method_idx(FUTURE, p_fget, "get");
    let p_lget = b.proto_idx("Ljava/lang/Object;", vec!["I".into()]);
    let m_lget = b.method_idx(LIST, p_lget, "get");
    let p_size = b.proto_idx("I", vec![]);
    let m_size = b.method_idx(LIST, p_size, "size");
    let p_info_state = b.proto_idx(STATE, vec![]);
    let m_info_state = b.method_idx(WORK_INFO, p_info_state, "getState");
    let p_finished = b.proto_idx("Z", vec![]);
    let m_finished = b.method_idx(STATE, p_finished, "isFinished");
    let f_succ = b.field_idx(STATE, STATE, "SUCCEEDED");
    let s_tag = b.intern("sync");

    // enqueue: dois pedidos com tag "sync".
    let enq_code = enqueue_tagged_code(&mut b);
    let enq_code2 = enq_code.clone();
    let enq = b.code(6, 1, 3, enq_code);
    b.direct(
        probe,
        "enqueue",
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(enq),
    );
    let enq2 = b.code(6, 1, 3, enq_code2);
    b.direct(
        probe,
        "enqueue2",
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(enq2),
    );

    // size(ctx): tamanho da lista de WorkInfo por tag.
    // v0 wm, v1 tag, v2 future, v3 lista, v4 tamanho, v5 ctx (regs=6, ins=1).
    let mut u = Vec::new();
    u.extend(op35c(0x71, 1, m_get, [5, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 0));
    u.extend(op21c(0x1a, 1, s_tag as u16));
    u.extend(op35c(0x6e, 2, m_by_tag, [0, 1, 0, 0, 0]));
    u.extend(op11x(0x0c, 2));
    u.extend(op35c(0x72, 1, m_fget, [2, 0, 0, 0, 0])); // invoke-interface Future.get
    u.extend(op11x(0x0c, 3));
    u.extend(op35c(0x72, 1, m_size, [3, 0, 0, 0, 0])); // invoke-interface List.size
    u.extend(op11x(0x0a, 4)); // move-result v4
    u.extend(op11x(0x0f, 4)); // return v4
    let size_code = b.code(6, 1, 3, u);
    b.direct(
        probe,
        "size",
        "I",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(size_code),
    );

    // firstIsSucceeded(ctx): WorkInfo[0].getState() == WorkInfo.State.SUCCEEDED
    // (identidade). v0 wm, v1 tag, v2 future, v3 lista, v4 índice, v5 info,
    // v6 estado, v7 ctx (regs=8, ins=1).
    let mut u = Vec::new();
    u.extend(op35c(0x71, 1, m_get, [7, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 0));
    u.extend(op21c(0x1a, 1, s_tag as u16));
    u.extend(op35c(0x6e, 2, m_by_tag, [0, 1, 0, 0, 0]));
    u.extend(op11x(0x0c, 2));
    u.extend(op35c(0x72, 1, m_fget, [2, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 3));
    u.extend(op11n(0x12, 4, 0)); // const/4 v4, 0
    u.extend(op35c(0x72, 2, m_lget, [3, 4, 0, 0, 0])); // List.get(0)
    u.extend(op11x(0x0c, 5));
    u.extend(op35c(0x6e, 1, m_info_state, [5, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 6));
    u.extend(op21c(0x62, 5, f_succ)); // sget-object v5, State.SUCCEEDED
    u.extend(op22t(0x33, 5, 6, 4)); // if-ne v5, v6 -> L0 (4 unidades adiante)
    u.extend(op11n(0x12, 7, 1)); // const/4 v7, 1
    u.extend(op11x(0x0f, 7)); // return v7
    u.extend(op11n(0x12, 7, 0)); // L0: const/4 v7, 0
    u.extend(op11x(0x0f, 7));
    let succ_code = b.code(8, 1, 3, u);
    b.direct(
        probe,
        "firstIsSucceeded",
        "I",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(succ_code),
    );

    // firstIsFinished(ctx): WorkInfo[0].getState().isFinished()
    let mut u = Vec::new();
    u.extend(op35c(0x71, 1, m_get, [7, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 0));
    u.extend(op21c(0x1a, 1, s_tag as u16));
    u.extend(op35c(0x6e, 2, m_by_tag, [0, 1, 0, 0, 0]));
    u.extend(op11x(0x0c, 2));
    u.extend(op35c(0x72, 1, m_fget, [2, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 3));
    u.extend(op11n(0x12, 4, 0));
    u.extend(op35c(0x72, 2, m_lget, [3, 4, 0, 0, 0]));
    u.extend(op11x(0x0c, 5));
    u.extend(op35c(0x6e, 1, m_info_state, [5, 0, 0, 0, 0]));
    u.extend(op11x(0x0c, 6));
    u.extend(op35c(0x6e, 1, m_finished, [6, 0, 0, 0, 0]));
    u.extend(op11x(0x0a, 7));
    u.extend(op11x(0x0f, 7));
    let fin_code = b.code(8, 1, 3, u);
    b.direct(
        probe,
        "firstIsFinished",
        "I",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(fin_code),
    );

    let mut e = engine_of(&b);
    let c = [Value::Null];
    e.invoke_static("LWiProbe;", "enqueue", "(Landroid/content/Context;)V", &c)
        .expect("enqueue");
    e.invoke_static("LWiProbe;", "enqueue2", "(Landroid/content/Context;)V", &c)
        .expect("enqueue2");
    assert_eq!(e.m8_work().len(), 2);
    let size = e
        .invoke_static("LWiProbe;", "size", "(Landroid/content/Context;)I", &c)
        .expect("size");
    assert_eq!(size, Value::Int(2), "dois WorkInfo com a tag");
    let first_id = e.m8_work()[0].id.clone();
    e.m8_complete_work(&first_id);
    let succ = e
        .invoke_static(
            "LWiProbe;",
            "firstIsSucceeded",
            "(Landroid/content/Context;)I",
            &c,
        )
        .expect("firstIsSucceeded");
    assert_eq!(
        succ,
        Value::Int(1),
        "getState() é o mesmo objeto de WorkInfo.State.SUCCEEDED"
    );
    let fin = e
        .invoke_static(
            "LWiProbe;",
            "firstIsFinished",
            "(Landroid/content/Context;)I",
            &c,
        )
        .expect("firstIsFinished");
    assert_eq!(fin, Value::Int(1), "SUCCEEDED.isFinished() == true");
}

const EXISTING_POLICY: &str = "Landroidx/work/ExistingWorkPolicy;";

/// Um `enqueueUniqueWork("daily", policy, Builder(SyncWorker).build())` em bytecode.
/// Registros: v0 builder, v1 Class, v2 nome, v3 pedido, v4 WorkManager, v5 política,
/// v6 Context (regs=7, ins=1).
fn unique_enqueue_block(u: &mut Vec<u16>, ids: &UniqueIds, policy_field: u16, name_idx: u32) {
    u.extend(op21c(0x22, 0, ids.t_builder)); // new-instance v0, Builder
    u.extend(op21c(0x1c, 1, ids.t_worker)); // const-class v1, SyncWorker
    u.extend(op35c(0x70, 2, ids.m_init, [0, 1, 0, 0, 0])); // invoke-direct {v0,v1} <init>
    u.extend(op35c(0x6e, 1, ids.m_build, [0, 0, 0, 0, 0])); // invoke-virtual {v0} build
    u.extend(op11x(0x0c, 3)); // move-result-object v3
    u.extend(op21c(0x1a, 2, name_idx as u16)); // const-string v2, "daily"
    u.extend(op21c(0x62, 5, policy_field)); // sget-object v5, ExistingWorkPolicy.X
    u.extend(op35c(0x6e, 4, ids.m_uniq, [4, 2, 5, 3, 0])); // invoke-virtual {v4,v2,v5,v3}
}

struct UniqueIds {
    t_builder: u16,
    t_worker: u16,
    m_init: u16,
    m_build: u16,
    m_uniq: u16,
    m_get: u16,
}

#[test]
fn dex_unique_work_keep_and_replace_follow_existing_work_policy() {
    let mut b = DexBuilder::new();
    let probe = b.class("LUniqueProbe;", "Ljava/lang/Object;");
    let p_get = b.proto_idx(WM, vec!["Landroid/content/Context;".into()]);
    let p_init = b.proto_idx("V", vec!["Ljava/lang/Class;".into()]);
    let p_build = b.proto_idx(REQUEST, vec![]);
    let p_uniq = b.proto_idx(
        "Landroidx/work/Operation;",
        vec![
            "Ljava/lang/String;".into(),
            EXISTING_POLICY.into(),
            REQUEST.into(),
        ],
    );
    let ids = UniqueIds {
        t_builder: b.type_idx(BUILDER),
        t_worker: b.type_idx(WORKER),
        m_init: b.method_idx(BUILDER, p_init, "<init>"),
        m_build: b.method_idx(BUILDER, p_build, "build"),
        m_uniq: b.method_idx(WM, p_uniq, "enqueueUniqueWork"),
        m_get: b.method_idx(WM, p_get, "getInstance"),
    };
    let f_keep = b.field_idx(EXISTING_POLICY, EXISTING_POLICY, "KEEP");
    let f_replace = b.field_idx(EXISTING_POLICY, EXISTING_POLICY, "REPLACE");
    let name_idx = b.intern("daily");

    // run(ctx): KEEP, KEEP, REPLACE sobre o mesmo nome único.
    let mut u = Vec::new();
    u.extend(op35c(0x71, 1, ids.m_get, [6, 0, 0, 0, 0])); // invoke-static {v6} getInstance
    u.extend(op11x(0x0c, 4)); // move-result-object v4
    unique_enqueue_block(&mut u, &ids, f_keep, name_idx);
    unique_enqueue_block(&mut u, &ids, f_keep, name_idx);
    unique_enqueue_block(&mut u, &ids, f_replace, name_idx);
    u.extend(op10x(0x0e)); // return-void
    let code = b.code(7, 1, 3, u);
    b.direct(
        probe,
        "run",
        "V",
        vec!["Landroid/content/Context;"],
        ACC_PUBLIC | ACC_STATIC,
        Some(code),
    );

    let mut e = engine_of(&b);
    e.invoke_static(
        "LUniqueProbe;",
        "run",
        "(Landroid/content/Context;)V",
        &[Value::Null],
    )
    .expect("run");

    let work = e.m8_work();
    assert_eq!(
        work.len(),
        2,
        "KEEP descarta o segundo pedido; REPLACE entra com o terceiro"
    );
    assert_eq!(work[0].state, "CANCELLED", "REPLACE cancela o enfileirado");
    assert_eq!(
        work[1].state, "ENQUEUED",
        "pedido do REPLACE fica enfileirado"
    );
    assert_eq!(work[0].unique_name.as_deref(), Some("daily"));
    assert_eq!(work[1].unique_name.as_deref(), Some("daily"));
    assert_ne!(work[0].id, work[1].id, "cada pedido tem o seu UUID");
}
