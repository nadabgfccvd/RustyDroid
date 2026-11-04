//! Monitores mantidos são raízes do GC: um objeto travado com `monitor-enter`
//! sobrevive a uma coleta mesmo sem nenhuma outra referência; após o
//! `monitor-exit` pareado ele volta a ser coletável (RAM liberada).

use rd_testsupport::vm::*;
use rd_vm::{gc::FREED_CLASS, heap::HeapObj, Engine, Value};

const LOCK_CLS: &str = "LLock;";
const RUN_CLS: &str = "LMonGc;";

/// `static run()Ljava/lang/Object;`: `new-instance v0, LLock;`, depois `extra`,
/// depois `return-object v0`. O objeto só é alcançável pelo valor de retorno.
fn engine_returning_lock(extra: Vec<u16>) -> Engine {
    let mut b = DexBuilder::new();
    let cls = b.class(RUN_CLS, "Ljava/lang/Object;");
    let _lock = b.class(LOCK_CLS, "Ljava/lang/Object;");
    let lock_t = b.type_idx(LOCK_CLS);
    let mut units = op21c(0x22, 0, lock_t); // new-instance v0, LLock;
    units.extend(extra);
    units.extend(op11x(0x11, 0)); // return-object v0
    b.direct(
        cls,
        "run",
        "Ljava/lang/Object;",
        vec![],
        ACC_PUBLIC | ACC_STATIC,
        Some(b.code(1, 0, 0, units)),
    );
    engine_of(&b)
}

fn returned_obj(e: &mut Engine) -> u32 {
    // Os primeiros slots (ids baixos) podem ser pinados pela varredura
    // conservadora de tokens `Debug` do estado do framework; o padding evita que
    // o teste passe ou falhe por esse motivo, e não pelo monitor.
    for _ in 0..1000 {
        e.heap.alloc_string(String::new()).unwrap();
    }
    match e.invoke_static(RUN_CLS, "run", "()Ljava/lang/Object;", &[]) {
        Ok(Value::Obj(r)) => r,
        other => panic!("esperava objeto, veio {other:?}"),
    }
}

fn class_of(e: &Engine, r: u32) -> String {
    match e.heap.get(r).unwrap() {
        HeapObj::Instance { class, .. } => class.to_string(),
        other => panic!("esperava instância, veio {other:?}"),
    }
}

#[test]
fn held_monitor_keeps_its_object_alive_across_gc() {
    let mut e = engine_returning_lock(op11x(0x1D, 0)); // monitor-enter v0
    let r = returned_obj(&mut e);
    e.gc();
    assert_ne!(
        class_of(&e, r),
        FREED_CLASS,
        "objeto travado foi coletado: a tabela de monitores não é raiz"
    );
}

#[test]
fn released_monitor_lets_its_object_be_collected() {
    let mut e = engine_returning_lock(
        [op11x(0x1D, 0), op11x(0x1E, 0)] // monitor-enter v0; monitor-exit v0
            .into_iter()
            .flatten()
            .collect(),
    );
    let r = returned_obj(&mut e);
    e.gc();
    assert_eq!(
        class_of(&e, r),
        FREED_CLASS,
        "monitor liberado ainda segura o objeto (vazamento de RAM)"
    );
}
