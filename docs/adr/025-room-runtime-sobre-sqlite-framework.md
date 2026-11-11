# ADR 025 — Runtime do Room no VM: `init` real sobre o SQLite de framework

- **Status:** aceito na etapa 1 (caminho A1, mergeado com este PR; precedente ADR 008). Etapas seguintes pendentes.
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

1. Caminho **A**, em etapas, cada uma com teste de regressão (convenção do RD Sentinel, retirado pela ADR 010: mudança de VM):
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

## Resultado do spike da etapa 1 (2026-11-03, não mergeado)

O spike está no branch `wip/adr025-step1-spike` (commit `299b464`). Ele não foi mergeado e não deve ser.

O que o spike fez:

- O builder guarda o `Context` e o `openHelperFactory` em campos do próprio objeto do builder. Assim o GC os rastreia, sem mapa lateral.
- `build()` cria um `DatabaseConfiguration` real, grava os campos pelo nome e chama `init(config)` no `_Impl` do DEX. Os campos que o builder não modela recebem o padrão do Room: `callbacks`, `autoMigrationSpecs`, `typeConverters`, `migrationNotRequiredFrom`, `migrationContainer`, executores (`Executors.newFixedThreadPool`), `queryCoroutineContext` (`EmptyCoroutineContext`), `allowMainThreadQueries` e `preparedStatementCacheSize`.
- Intrínseco: `kotlin.jvm.internal.Reflection.getOrCreateKotlinClass(Class)` devolve `ClassReference`, como o stdlib faz sem kotlin-reflect.

Resultado no F-Droid: o `lateinit connectionManager` deixou de acontecer. Mas a cadeia para em outro ponto:

1. `Reflection.typeOf(Class)` cai em `ReflectionFactoryImpl` (kotlin-reflect), que o VM não executa.
2. `RoomDatabase.init` cria `CoroutineScope(SupervisorJob(...))`. O kotlinx-coroutines entra em `JobSupport`, cujo campo estático `$FU` é um `AtomicReferenceFieldUpdater.newUpdater(...)`. Esse objeto é construído por reflexão, e o VM o entrega como null. A exceção final é `NullPointerException` em `getClass`.

A regra 2 desta ADR manda parar quando a etapa 1 exigir coroutines além do que o VM tem. Por isso a implementação para aqui, e a decisão volta ao dono.

**Decisão necessária:**

- **A1. Continuar o caminho A com intrínsecos.** Implementar no host `AtomicReferenceFieldUpdater.newUpdater`, `Dispatchers.getDefault`, `ExecutorsKt.from` e o que a cadeia exigir depois (`StateFlow` e outros). Cada um é pequeno, mas a lista ainda não é conhecida. Cada passo tem teste e medição com a CLI no F-Droid.
- **B. Modelar `RoomConnectionManager` no host** (contingência desta ADR). Exige ADR própria, conforme o item 2 da decisão.

Recomendação de quem escreveu o spike: A1 com um limite de passos. Se a cadeia passar de uns poucos intrínsecos de coroutines, abrir a ADR de B. O spike não altera código de biblioteca do Room, e cada intrínseco fica isolado em `intrinsics.rs`.

## Resultado da etapa 1 (caminho A1, 2026-11-03)

O dono mandou seguir todos os passos. Executado A1, com limite de passos. Nenhum intrínseco de coroutines foi necessário, e a contingência B não foi acionada.

**Causa real do ponto de parada do spike.** O spike passava `queryCoroutineContext = EmptyCoroutineContext` ao `DatabaseConfiguration`. No Room, esse campo é `null` quando o builder não recebe contexto, e é esse `null` que leva `RoomDatabase.init` ao `queryExecutor`. Com um contexto não nulo, o init entra no ramo do dispatcher. Ali `EmptyCoroutineContext.get(Key)` devolve `const/4 0` como objeto, que o VM guarda como `Int(0)`, e o `invoke-virtual getClass` falha com NPE. O NPE não vinha do `JobSupport`. O campo `$FU` do `JobSupport` foi lido 313 vezes no trace e nunca veio nulo.

**Cadeia do F-Droid (`rd app run --trace`), uma causa por passo:**

1. `RoomDatabase.init` com `queryCoroutineContext` nulo, como o Room faz por padrão (`m8.rs`, `room_init`).
2. `WorkRequest$Builder`: os setters fluentes (`setConstraints`, ...) devolviam null, e o `build()` seguinte falhava. Agora devolvem o próprio builder quando o retorno é `*$Builder;` (`sig_returns_builder`).
3. `PeriodicWorkRequest$Builder.build()` devolvia `WorkRequest`, e o app faz `check-cast` para `PeriodicWorkRequest`. Agora devolve o subtipo (`work_request_product`).
4. `NotificationChannel.setGroup(null)` falhava. No Android, `null` é válido e remove o grupo (`m9/notifications.rs`).
5. `NotificationChannel.setShowBadge`, `canShowBadge`, `setLightColor`, `getLightColor` não existiam. Agora existem, com `show_badge` padrão `true`, como no Android. `setSound`, `setVibrationPattern` e `setConversationId` aceitam o argumento e não guardam estado, porque o host não toca áudio nem vibra.

**Estado depois da etapa 1.** `App.onCreate` do F-Droid roda até o fim. O WorkManager enfileira o trabalho periódico de repositório, com o `WorkDatabase` (Room) inicializado de verdade sobre o SQLite de framework. A falha seguinte é de Compose (`AbstractComposeView.ensureCompositionCreated`, "Required value was null"), na tela de entrada. Isso está fora do escopo do ADR 025 e fica como próxima frente.

> **Atualização (2026-11-03, bloco 9):** o F-Droid real também conclui `App.onCreate` sem exceção. O teste `fdroid_oncreate_e2e` passa. Para isso, o caminho do Ktor exigiu metadados de `Class` (ADR 027). Essa parte não é ADR 025 (Room/WorkManager), mas fica registrada aqui porque o teste é deste ADR. A primeira tela do F-Droid também renderiza (`rd app shot`) depois que a CLI passou a rodar numa thread de 32 MiB, como `app run` já fazia; na thread principal de 8 MiB o draw do Compose estourava o orçamento de pilha nativa e a tela saía em branco.

**Regressão.** `crates/rd-cli/tests/fdroid_oncreate_e2e.rs` falha se `App.onCreate`, o lateinit do Room ou o `scheduleOrCancel` voltarem a lançar exceção. Os testes unitários de `m8.rs` cobrem os helpers de builder.

**Limites desta etapa.** Não foi medida invalidação de tabelas nem transação do Room sobre o VM. Layering (ADR 021, etapa 1 "sessions") não foi iniciado.
