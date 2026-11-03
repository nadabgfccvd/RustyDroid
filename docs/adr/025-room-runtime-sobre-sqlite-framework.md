# ADR 025 — Runtime do Room no VM: `init` real sobre o SQLite de framework

- **Status:** proposto (aceito quando mergeado, precedente ADR 008)
- **Data:** 2026-11-03 · **Escopo:** `crates/rd-vm/src/m8.rs` (`room_builder`), `crates/rd-vm/src/jsqlite.rs` (SQLite host, ADR 005) e `crates/rd-vm/src/jfile.rs` (VFS), roteamento de `RoomDatabase` no VM
- **Relacionados:** ADR 003 (M8, host de compatibilidade), ADR 005 (SQLite em subprocesso), ADR 024 (estado por engine), F-Droid como golden de WorkManager/Room (`golden-apks/org.fdroid.fdroid.apk`)
- **Ordem:** ADR antes de código (CONTRIBUTING). Este PR é só documentação.

## Contexto

O F-Droid chega em `WorkManagerImpl.getInstance` e falha com `lateinit property connectionManager has not been initialized` (ver `docs/ROADMAP.md`, seção F-Droid). Achados verificados nos DEX do APK (`classes.dex`, `classes2.dex`, 2026-11-03):

- `RoomDatabase` declara `private connectionManager: RoomConnectionManager`. Os getters da classe leem o campo e lançam `UninitializedPropertyAccessException` se ele for nulo.
- O campo só é preenchido em `RoomDatabase.init(DatabaseConfiguration)`, que chama `createConnectionManager(config)`.
- Hoje `room_builder` (`m8.rs`) instancia `<DB>_Impl` diretamente e **não** chama `init`. Por isso o campo fica nulo quando o `WorkDatabase` é usado na inicialização do WorkManager.
- O F-Droid **não referencia** `BundledSQLiteDriver` nem `AndroidSQLiteDriver`. Ele referencia `FrameworkSQLiteOpenHelperFactory` (3 ocorrências entre os DEX) e `SupportSQLiteOpenHelper` (14). Ou seja, o caminho do app é o SQLite de framework (`android.database.sqlite`), que o VM já modela pelo subprocesso da ADR 005.

## Opções

**A. Executar o `init` real, com o SQLite de framework como helper.** O `RoomDatabase.Builder.build()` roda no DEX (sem o atalho do `room_builder`). O `DatabaseConfiguration` vem do builder, o `SupportSQLiteOpenHelperFactory` é o de framework, e as operações SQL vão para o host SQLite (ADR 005).
- Vantagem: a semântica do Room fica no código do próprio Room (transações, invalidation tracker, migrações). É o que o app espera.
- Custo: a superfície do VM cresce. O Room 2.8 usa coroutines (`useConnection` é `suspend`), `Executor`, `ThreadLocal` e `Lock`. Parte disso já existe, mas não foi testada com Room.

**B. Modelar `RoomConnectionManager` no host.** Um objeto de host implementa a conexão, e o `init` continua sendo do host.
- Vantagem: pequena, e resolve o `lateinit` na hora.
- Desvantagem: duplica a lógica de transação e de invalidação do Room, que é justamente a parte que o app depende. Tende a divergir a cada versão do Room.

**C. Manter o atalho do `room_builder` e pré-inicializar o WorkManager.** Um caminho especial só para `WorkDatabase`.
- Rejeitada como solução geral: cria semântica de WorkManager diferente da real. Pode ser usada como ponte curta, desde que marcada como tal em `compat_gaps`.

## Decisão proposta

1. Caminho **A**, em etapas, cada uma com teste de regressão (RD Sentinel: mudança de VM):
   1. Fazer `room_builder` chamar `init` com um `DatabaseConfiguration` real, usando o helper de framework. Teste: `RoomDatabase.connectionManager` não é nulo após `build()`, e um `SELECT` simples de um DAO devolve linhas.
   2. Suportar `runInTransaction`/`beginTransaction` e o invalidation tracker. Teste: uma escrita dispara a invalidação observada.
   3. Validar com o F-Droid: `WorkDatabase.systemIdInfoDao()` funciona sem exceção.
2. Se a etapa 1 exigir suporte a coroutines além do que o VM tem, **parar e revisar** a decisão. Nesse caso, B vira o plano de contingência, com ADR própria.
3. C não entra como caminho de produto. Se for usada como ponte, a entrada em `compat_gaps` é obrigatória.

## Consequências

- O ganho imediato é o F-Droid passar do `connectionManager`. O ganho de longo prazo é que apps com Room deixam de depender do atalho do host.
- Risco: a etapa 1 pode revelar bloqueios em coroutines. Esse risco é o motivo para a ordem das etapas e para o ponto de parada da etapa 2.
- Nenhum teste existente deve ser removido. O atalho do `room_builder` só sai quando a etapa 1 passar nos testes atuais de Room.

## Fora de escopo

- Migrações de schema de apps reais (depende de `init` funcionar primeiro).
- Room com `BundledSQLiteDriver` (nativo). Nenhum dos APKs de referência usa.
