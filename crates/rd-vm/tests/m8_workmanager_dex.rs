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
