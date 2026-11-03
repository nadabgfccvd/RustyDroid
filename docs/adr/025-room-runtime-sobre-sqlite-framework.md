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

## Achados da etapa 1 (2026-11-03, leitura do DEX, sem código)

Verificados no `classes.dex`/`classes2.dex` do F-Droid (`golden-apks/org.fdroid.fdroid.apk`) com baksmali 2.5.2, e no código de `m8.rs`:

1. **Dois caminhos em `RoomDatabase.createConnectionManager$room_runtime`** (`RoomDatabase.smali`, linha 487). Se `createOpenDelegate()` lança `NotImplementedError` (o padrão), o engine usa `RoomConnectionManager(config, Function1, Function2)`, que depende de `config.sqliteDriver`. Se o `_Impl` sobrescreve `createOpenDelegate()`, o engine usa `RoomConnectionManager(config, RoomOpenDelegate, Function2)`, que lê `config.sqliteOpenHelperFactory`, `context`, `name`, `callbacks` e `journalMode`.
2. **O F-Droid usa o segundo caminho.** `WorkDatabase_Impl` sobrescreve `createOpenDelegate()` (`WorkDatabase_Impl.smali`, linha 443). O padrão do builder é `FrameworkSQLiteOpenHelperFactory` (`RoomDatabase$Builder.smali`, linha 317). Isso confirma o achado original: não há `BundledSQLiteDriver` no caminho.
3. **O atalho atual descarta a configuração.** `m8::room_builder` (`m8.rs`, perto da linha 2661) guarda só `(dbcls, dbname)`. `build()` aloca o `_Impl` e não chama `init`. `openHelperFactory`, `addCallback`, `addMigrations`, `setQueryExecutor`, `setTransactionExecutor` e `setJournalMode` devolvem `this` sem guardar o valor.
4. **`useConnection` é `suspend`.** `RoomConnectionManager.smali` recebe `Continuation` e não referencia `kotlinx.coroutines` diretamente. Ainda não foi verificado como o VM executa essas chamadas. É o primeiro ponto a testar na etapa 1.

**Linha de base da CLI em `main` (e983e02), 2026-11-03:**

| APK | Resultado | Exit |
|---|---|---|
| `org.tasks.apk` | `MISSING_ENTRY` (`activity LAUNCHER`, sem `--activity`) | 1 |
| `org.schabi.newpipe.apk` | `NOT_IMPLEMENTED` (`Collections.binarySearch`) | 2 |
| `org.fdroid.fdroid.apk` | `VM_EXCEPTION`: `lateinit property connectionManager has not been initialized` | 1 |

**Plano concreto da etapa 1** (substitui a descrição genérica acima):

- a. Trocar a tupla de `room_builders` por uma struct que guarda contexto, nome, `openHelperFactory`, callbacks, journal mode, executores, `allowMainThreadQueries` e migrations.
- b. Em `build()`, criar uma instância DEX de `Landroidx/room/DatabaseConfiguration;` e preencher os campos pelo nome (`heap.put_field`), sem chamar o construtor de muitos parâmetros. Depois chamar `init(config)` no `_Impl`.
- c. Teste de regressão (`m8`): após `build()`, `connectionManager` não é nulo. Teste de integração com o F-Droid: `WorkDatabase.systemIdInfoDao()` roda sem exceção, ou falha com uma causa nova e registrada.
- d. Se o `useConnection` exigir mais coroutines do que o VM tem, parar e revisar, como o item 2 da decisão manda.
