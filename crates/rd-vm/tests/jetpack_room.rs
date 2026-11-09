//! Apêndice D (ST-02) — Room 2.6.1 real sobre o VM: o `_Impl` gerado pelo
//! processador do Room roda no DEX, sobre o SQLite real (`jsqlite`).
//!
//! O DEX é gerado por `scripts/gen-room-fixture.sh` a partir de
//! `tests/fixtures/room/src` (javac + processador do Room 2.6.1, depois D8).
//! Não é versionado (`.dex` é bloqueante no RD Sentinel, ver ADR 036): rode
//! `scripts/gen-room-fixture.sh` e depois
//! `RD_ROOM_FIXTURE_DEX=target/room-fixture/classes.dex cargo test -p rd-vm --test jetpack_room -- --ignored`.
//!
//! Comportamento esperado é o do Room real (`RoomOpenHelper.onUpgrade`):
//! - migração 1→2 declarada: aplica o `ALTER TABLE` e preserva a linha;
//! - sem migração e sem `fallbackToDestructiveMigration`: lança
//!   `IllegalStateException` ("A migration from 1 to 2 was required but not found").
//! - um banco aberto, fechado e reaberto no mesmo processo não reclama da tabela
//!   TEMP do invalidation tracker (o `close` libera a conexão, como no Android).
//! - `fallbackToDestructiveMigrationFrom(1)`: sem migração 1→2, recria a tabela.
//! - `fallbackToDestructiveMigrationOnDowngrade()`: downgrade 2→1 recria a tabela.
//! - downgrade 2→1 sem fallback: recusa a abertura com `IllegalStateException`.

use rd_vm::engine::VmConfig;
use rd_vm::value::Value;
use rd_vm::Engine;

const V1: &str = "Lcom/example/roomfx/V1;";
const V2: &str = "Lcom/example/roomfx/V2;";
const SEED_SIG: &str = "(Landroid/content/Context;Ljava/lang/String;)I";
const OPEN_SIG: &str = "(Landroid/content/Context;Ljava/lang/String;)Ljava/lang/String;";

fn engine() -> Engine {
    let path = std::env::var("RD_ROOM_FIXTURE_DEX")
        .expect("defina RD_ROOM_FIXTURE_DEX (veja scripts/gen-room-fixture.sh)");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("ler {path}: {e}"));
    let dex = rd_dex::Dex::parse(bytes).expect("DEX do fixture parseia");
    Engine::new(vec![dex], VmConfig::default())
}

/// `Context` de teste, como nos demais testes do rd-vm.
fn context(e: &mut Engine) -> Value {
    Value::Obj(
        e.heap
            .alloc_instance("Landroid/content/Context;".into(), Vec::new())
            .unwrap(),
    )
}

fn name(e: &mut Engine, db: &str) -> Value {
    Value::Obj(e.heap.alloc_string(db.to_string()).unwrap())
}

fn call_string(e: &mut Engine, class: &str, method: &str, args: &[Value]) -> String {
    match e
        .invoke_static(class, method, OPEN_SIG, args)
        .unwrap_or_else(|err| panic!("{class}.{method}: {err:?}"))
    {
        Value::Obj(o) => e.heap.as_str(o).unwrap().to_string(),
        other => panic!("{class}.{method} devolveu {other:?}"),
    }
}

fn seed_v1(e: &mut Engine, db: &str) {
    let ctx = context(e);
    let n = name(e, db);
    let r = e
        .invoke_static(V1, "seed", SEED_SIG, &[ctx, n])
        .expect("seed v1");
    assert_eq!(r, Value::Int(1), "v1 gravou a linha");
}

#[test]
#[ignore = "exige RD_ROOM_FIXTURE_DEX (scripts/gen-room-fixture.sh)"]
fn migration_declared_runs_alter_and_keeps_rows() {
    let mut e = engine();
    seed_v1(&mut e, "a.db");
    let ctx = context(&mut e);
    let n = name(&mut e, "a.db");
    let got = call_string(&mut e, V2, "withMigration", &[ctx, n]);
    assert_eq!(got, "count=1,qty=0", "a migração 1→2 preserva a linha");
}

#[test]
#[ignore = "exige RD_ROOM_FIXTURE_DEX (scripts/gen-room-fixture.sh)"]
fn without_migration_refuses_to_open_like_room() {
    let mut e = engine();
    seed_v1(&mut e, "b.db");
    let ctx = context(&mut e);
    let n = name(&mut e, "b.db");
    let got = call_string(&mut e, V2, "withoutMigration", &[ctx, n]);
    assert!(
        got.starts_with(
            "error:java.lang.IllegalStateException:A migration from 1 to 2 was required but not found"
        ),
        "sem migração e sem fallback, o Room recusa a abertura: {got}"
    );
}

#[test]
#[ignore = "exige RD_ROOM_FIXTURE_DEX (scripts/gen-room-fixture.sh)"]
fn fresh_database_reopens_in_same_process() {
    let mut e = engine();
    for _ in 0..2 {
        let ctx = context(&mut e);
        let n = name(&mut e, "c.db");
        let got = call_string(&mut e, V2, "withoutMigration", &[ctx, n]);
        assert_eq!(
            got, "count=0",
            "reabrir após close não reclama da TEMP table"
        );
    }
}

fn seed_v2(e: &mut Engine, db: &str) {
    let ctx = context(e);
    let n = name(e, db);
    let r = e
        .invoke_static(V2, "seed", SEED_SIG, &[ctx, n])
        .expect("seed v2");
    assert_eq!(r, Value::Int(1), "v2 gravou a linha");
}

#[test]
#[ignore = "exige RD_ROOM_FIXTURE_DEX (scripts/gen-room-fixture.sh)"]
fn fallback_from_listed_version_recreates_the_table() {
    let mut e = engine();
    seed_v1(&mut e, "f.db");
    let ctx = context(&mut e);
    let n = name(&mut e, "f.db");
    let got = call_string(&mut e, V2, "withFallbackFrom", &[ctx, n]);
    assert_eq!(
        got, "count=0",
        "fallbackToDestructiveMigrationFrom(1) dispensa a migração 1→2 e recria a tabela vazia"
    );
}

#[test]
#[ignore = "exige RD_ROOM_FIXTURE_DEX (scripts/gen-room-fixture.sh)"]
fn downgrade_with_fallback_recreates_the_table() {
    let mut e = engine();
    seed_v2(&mut e, "g.db");
    let ctx = context(&mut e);
    let n = name(&mut e, "g.db");
    let got = call_string(&mut e, V1, "openOverV2", &[ctx, n]);
    assert_eq!(
        got, "count=0",
        "fallbackToDestructiveMigrationOnDowngrade recria a tabela da v1 sem as linhas da v2"
    );
}

#[test]
#[ignore = "exige RD_ROOM_FIXTURE_DEX (scripts/gen-room-fixture.sh)"]
fn downgrade_without_fallback_refuses_to_open_like_room() {
    let mut e = engine();
    seed_v2(&mut e, "h.db");
    let ctx = context(&mut e);
    let n = name(&mut e, "h.db");
    let got = call_string(&mut e, V1, "openOverV2WithoutFallback", &[ctx, n]);
    assert!(
        got.starts_with(
            "error:java.lang.IllegalStateException:A migration from 2 to 1 was required but not found"
        ),
        "sem fallback de downgrade, o Room recusa a abertura: {got}"
    );
}
