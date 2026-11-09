# Apêndices D + E — registro executável e ledger de conclusão honesta

Este documento é o ledger dos Apêndices D (AndroidX/Jetpack, shims por
módulo) e E (Google Play Services / Firebase, modo "sem GMS" honesto) da
spec (`RustyDroid.md`). Como no Apêndice C, a fonte executável da verdade
**não é este texto**: são `data/jetpack.toml` e `data/gms.toml`, validados
por `rd-framework::fd_jetpack`/`fd_gms`, consultáveis via
`rd jetpack`/`rd gms` e regenerados em `docs/jetpack.md`/`docs/gms.md`
por `cargo xtask jetpack-report|gms-report`.

## Semântica de status (idêntica à do Apêndice C)

- `supported` — modelo host com estado para o subconjunto documentado em
  `notes`/`key_apis`, com testes. **Nunca** significa a biblioteca ou o
  serviço inteiro.
- `partial` — um subconjunto real executa: DEX da biblioteca sobre a
  plataforma host e/ou shim host com estado. O restante cai em DEX,
  `NOT_FOUND` tipado ou desvio documentado.
- `stub` — só reconhecimento: hierarquia de tipos, constantes, inflate ou
  alocação sem comportamento. Presença em DEX não é cobertura.
- `not_implemented` — nenhum modelo registrado; a chamada falha de forma
  tipada (`NOT_IMPLEMENTED`/`NOT_FOUND`), nunca com sucesso fingido.

Nenhum status sobe sem código + teste no mesmo PR (regra do Apêndice C).

## Estratégias do Apêndice D

Bibliotecas AndroidX e de terceiros **viajam no APK** (DEX). Por isso cada
módulo declara uma estratégia:

- `dex-passthrough` — o DEX da biblioteca executa na VM; o host só fornece
  a plataforma `android.*`/`java.*` embaixo. É a estratégia correta para
  bibliotecas puras (Gson, Moshi, coroutines…); o status mede quanto da
  plataforma embaixo + dos pontos de integração funciona de verdade.
- `host-shim` — o host intercepta métodos listados com um modelo com
  estado (ex.: RecyclerView e WorkManager no M8; LiveData/ViewModelStore
  como fallback quando o APK não traz o corpo DEX). Fora da lista, o
  dispatch devolve `None` e o interpretador segue para o DEX real.
- `host-native` — implementação nativa no host, sem DEX esperado.

Jetpack Compose (UI-05) continua experimental/longo prazo na PARTE 0 da
spec: o registro o mantém `not_implemented` com gap explícito — registrar
não é implementar.

## Modo "sem GMS" honesto (Apêndice E)

Não há processo GMS, conta Google nem backend real — e o registro diz
isso em toda linha (`network = "none"`, validado):

- As APIs **existem** como shims host com respostas estruturadas,
  determinísticas e **programáveis pelo agente** (`Engine::gms_*`):
  localização fixa, usuário fake, catálogo de billing, token FCM etc.
- Eventos são **capturados** e consultáveis (Analytics, Crashlytics —
  incluindo crashes reais da VM —, ads, compras), nunca enviados à rede.
- Firestore/RTDB são stores locais em memória (record/replay NET-03):
  writes disparam listeners sincronamente, sem sincronização externa.
- `GoogleApiAvailability` é programável; o default `SUCCESS` significa
  apenas "o shim responde", jamais "o Google respondeu".

## Como atualizar

1. Mexeu num shim AndroidX/GMS? Atualize a linha em
   `data/jetpack.toml`/`data/gms.toml` **no mesmo PR**.
2. Rode `rd jetpack check --json` / `rd gms check --json` (ou os testes
   de `rd-framework`).
3. Regenere relatórios e spec: `cargo xtask jetpack-report jetpack-spec
   gms-report gms-spec`.

## Ledger de execução (branch `appendix-d-e-20261025`)

Estado por etapa: ver a tabela deste documento e o histórico dos PRs do trilho D/E (#213, #214); o antigo `PROGRESS-appendix-de.md` (log de sessão) saiu do repositório — está no histórico do git.
Os PRs #191/#192 (Apêndice C, outro agente) não são tocados por este
trabalho; arquivos compartilhados recebem, no máximo, edições aditivas
mínimas documentadas no PR.

- **S6 (2026-10-25, branch `appendix-e-gms-runtime-20261025`)**: o
  runtime do Apêndice E vive em `crates/rd-vm/src/gms.rs` (host-first,
  estado em `HostState.gms`, dispatch aditivo em `interp.rs` ao lado do
  Apêndice D) com 21 testes em `crates/rd-vm/tests/gms.rs`. 20 dos 21
  serviços sobem de `not_implemented` para `partial` (subconjuntos
  testados descritos em `data/gms.toml`); ML Kit permanece
  `not_implemented` e sem dispatch — o teste
  `unknown_gms_surface_returns_none` fixa essa fronteira.
- **S8 (rodada 1 de D/E, branch `appendix-d-e-round1-20261105`)**: inventário
  por item, sem subir status sem teste.
  - **D — stub → partial (3)**: `androidx-drawerlayout`,
    `androidx-swiperefreshlayout`, `androidx-cardview`. Shim host em
    `crates/rd-vm/src/androidx_widgets.rs` (ligado em `call_host_shim_layer`
    e em `native.rs`), 9 testes em `crates/rd-vm/tests/androidx_widgets.rs`.
    Não modelados (dito em `notes`): animação, validação de
    `layout_gravity`, sobrecargas com gravidade `int` (caem no DEX),
    `DrawerListener`, gestos de arrasto, `onRefresh` (o host não dispara),
    sombra/cor de fundo do CardView.
  - **D — continuam stub (4)**: `androidx-viewpager2`,
    `androidx-constraintlayout`, `androidx-coordinatorlayout`,
    `androidx-material`. Sem shim; nenhum status sobe nesta rodada.
  - **D — not_implemented (20)**: a estratégia de cada item já está
    registrada em `data/jetpack.toml`. Próximo passo planejado (sem
    mudança de status até haver teste):

    | Biblioteca | Estratégia | Próximo passo / bloqueio |
    |---|---|---|
    | activity | dex-passthrough | Ciclo onCreate→onResume via DEX sobre `android.app` |
    | annotation | dex-passthrough | Só metadado de compilação; decisão de status pendente |
    | paging | dex-passthrough | `PagingSource.load` depende de coroutines |
    | palette | dex-passthrough | `Palette.from(Bitmap)` depende de decode de bitmap (UI-02) |
    | compose | host-native | UI-05 experimental/longo prazo; segue `not_implemented` |
    | emoji2 | dex-passthrough | Fora do escopo desta rodada (fontes de emoji) |
    | media3 | dex-passthrough | Depende de MEDIA-01/02 (pipeline de mídia) |
    | camerax | host-shim → DEX real | `partial` (S19): o DEX real do CameraX 1.6.0 roda sobre a Camera2 virtual, sem shim novo. Preview renderiza. Captura não confirmada. Prova ponta a ponta local (APK fora do CI) |
    | security-crypto | host-shim | **Partial (S17, S20).** Depende de AndroidKeyStore (ST-04). Persistência entre sessões no VFS app-private (S20); chave do AndroidKeyStore em claro no VFS, sem hardware |
    | credentials | host-shim | **Partial (S18, S18b).** Padrão sem provedor. Provedor fake opt-in só para `get` (senha ou falha `Get*`); create sem armazenamento; `clear` |
    | photopicker | host-shim | Depende de URI programável para `PickVisualMedia` |
    | hilt-dagger | dex-passthrough | Exige app com DEX gerado pelo Hilt |
    | kotlinx-coroutines | dex-passthrough | Depende de `Dispatchers.Main` no modelo de threads |
    | rxjava | dex-passthrough | Depende de bridge de `Schedulers` |
    | retrofit | dex-passthrough | Depende de `java.lang.reflect.Proxy` |
    | moshi | dex-passthrough | DEX puro; sem ação host prevista |
    | gson | dex-passthrough | DEX puro dependente de reflexão |
    | coil | dex-passthrough | Depende de pipeline de imagens (UI-02) |
    | glide | dex-passthrough | Depende de pipeline de imagens (UI-02) |
    | picasso | dex-passthrough | Depende de pipeline de imagens (UI-02) |

  - **E — testes de erro e ciclo de vida (9 novos em
    `crates/rd-vm/tests/gms.rs`, suíte gms com 42 testes passando)**:
    - `storage_getbytes_size_limit_error_and_delete_lifecycle`
      (firebase-storage): limite menor que o blob falha sem alterar o
      blob; limite igual ou zero tem sucesso; `delete` → leitura falha →
      segundo `delete` falha.
    - `safetynet_programmed_jws_and_nonce_record_track_api_key`
      (gms-integrity): `gms_set_attestation_jws` é devolvido por
      `getJwsResult`; o registro de nonce distingue `apiKeyPresent`.
    - `google_signin_sign_out_ends_silent_session_and_sign_in_restores_it`
      (gms-auth): `gms_sign_out_google` faz `silentSignIn` falhar com
      código 4 e `getLastSignedInAccount` voltar a null; reautenticar
      restaura a sessão.
    - `location_fix_cleared_mid_session_returns_null_until_reseeded`
      (gms-location): limpar o fix devolve null em `getLastLocation`;
      reprogramar restaura.
    - `nearby_request_connection_only_reaches_discovered_endpoints`
      (gms-nearby): **correção de comportamento**. `requestConnection`
      concluía com sucesso para qualquer endpoint, inclusive um nunca
      descoberto. Agora só conclui para endpoint descoberto ou programado
      e falha com `ApiException` (ENDPOINT_UNKNOWN) para os demais. Texto
      em `data/gms.toml` e `docs/gms.md` regenerado.
    - `maps_marker_remove_drops_only_that_marker` (gms-maps): `remove`
      tira só o marcador alvo, e o outro permanece.
    - `fit_zero_steps_yields_empty_data_set_not_fake_points` (gms-fit-health):
      sem passos programados, o histórico devolve lista vazia, sem ponto
      inventado.
    - `analytics_null_user_property_value_removes_it` (firebase-analytics):
      valor null em `setUserProperty` remove a propriedade.
    - `firestore_missing_document_reads_as_absent_not_error`
      (firebase-firestore): `get` de documento inexistente conclui com
      sucesso e `exists() == false`, sem dados inventados.
  - **E — auditoria**: fechada na rodada S9 (ver inventário abaixo).

### S9 — Apêndice E: inventário final dos 20 serviços `partial` (2026-11-05)

Critério (plano §5): cada serviço `partial` precisa de teste de **sucesso**,
de **erro** e de **ciclo de vida**, quando o shim tem esses caminhos. Onde o
shim não tem caminho de erro programável, a célula diz isso em vez de inventar
um teste. Testes em `crates/rd-vm/tests/gms.rs` e `crates/rd-vm/tests/gms_erro_ciclo.rs` (etapa 7).

| Serviço | Sucesso | Erro | Ciclo de vida |
|---|---|---|---|
| gms-common | `availability_programmable_codes` | mesmo teste (códigos programados) | troca de código no mesmo teste |
| gms-tasks | `tasks_completion_source_roundtrip_with_dex_listener` | `tasks_failure_surfaces_exception` | `tasks_when_all_success_failure_and_complete` |
| gms-location | `location_fix_and_callback_delivery` | `location_failure_blocks_reads_and_registrations_then_recovers` (falha programada: leitura e registro falham) | mesmo teste (remoção segue disponível; recuperação volta a entregar); `location_fix_cleared_mid_session_returns_null_until_reseeded` |
| gms-maps | `maps_ready_camera_and_markers` | `maps_load_failure_blocks_map_ready_then_recovers` (falha programada: initialize com código, sem onMapReady) | mesmo teste (limpar a falha volta a entregar o mapa) |
| gms-billing | `billing_product_details_modern_path` | `billing_purchase_flow_and_cancel` (cancel) | `billing_listener_relaunching_flow_is_bounded` |
| gms-auth | `google_signin_silent_requires_host_completion` | ApiException 4 (mesmo teste) | `google_signin_sign_out_ends_silent_session_and_sign_in_restores_it` |
| gms-ads | `ads_banner_success_failure_and_rewarded` | `gms_ads_set_failure` (mesmo teste) | `clear_failure` no mesmo teste |
| gms-fit-health | `fit_steps_programmable` | `fit_failure_does_not_start_sessions` (falha sem efeito colateral) | `fit_failure_blocks_reads_and_writes_then_recovers` (bloqueio e recuperação); `fit_zero_steps_yields_empty_data_set_not_fake_points` |
| gms-cast | `cast_states_follow_programmed_session` | `cast_failed_connect_creates_no_session_then_recovers` (falha sem sessão; erro em `gms_cast_last_error`) | `cast_state_listener_fires_only_on_real_transitions` |
| gms-nearby | `nearby_discovery_reports_programmed_endpoint` | `nearby_request_connection_only_reaches_discovered_endpoints` | `nearby_lifecycle_discovery_and_connections` |
| gms-integrity | `integrity_and_safetynet_deterministic_tokens` | `integrity_failure_code_is_programmable_per_error_kind`; `safetynet_failure_blocks_attestation_then_recovers` (attest com ApiException, sem nonce) | `integrity_failure_then_recovery_is_a_full_lifecycle`; mesmo teste do SafetyNet (recuperação) |
| firebase-core | `firebase_autoinit_and_strict_mode` (auto-init) | strict sem initializeApp (mesmo teste) | mesmo teste (init → uso) |
| firebase-analytics | `firebase_autoinit_and_strict_mode` (evento) | `analytics_invalid_names_are_rejected_then_valid_events_recorded` (nome inválido rejeitado e contado) | mesmo teste (coleção desligada e religada) |
| firebase-auth | `firebase_auth_full_flow` | senha errada (mesmo teste) | `firebase_auth_state_listener_fires_on_add_and_signin` |
| firebase-messaging | `messaging_token_topics_and_delivery` | `messaging_failure_blocks_token_and_topics_then_recovers` (token e tópico falham com IOException; sem efeito) | mesmo teste (deleteToken segue; recuperação restaura token e assinatura) |
| firebase-crashlytics | `crashlytics_records_and_uncaught_bridge` | `crashlytics_custom_keys_capped_at_64_and_truncated` (limite de 64 chaves e truncamento) | crash não capturado vira registro; coleção desligada (mesmo teste) |
| firebase-firestore | `firestore_set_get_query_and_seed` | `firestore_missing_document_reads_as_absent_not_error` | `firestore_query_filter_and_snapshot_listener` |
| firebase-rtdb | `rtdb_set_resolve_and_listener` | `rtdb_permission_denied_cancels_listeners_and_blocks_writes_then_recovers` (negação: onCancelled, escrita não grava, remoção e leitura falham) | mesmo teste (listener registrado volta a receber após recuperação) |
| firebase-remoteconfig | `remoteconfig_fetch_activate_semantics` | `remote_config_failed_fetch_keeps_active_values_then_recovers` | fetch falho mantém valores ativos; recuperação aplica o novo (mesmo teste) |
| firebase-storage | `storage_seed_download_and_missing` | `storage_getbytes_size_limit_error_and_delete_lifecycle` | mesmo teste (delete) |

Na etapa 7, cada célula "sem caminho de erro" ganhou uma falha programável opt-in
(sem falha programada, o comportamento é o de antes), com teste que falha sem ela.
As células que ainda dizem "sem caminho de erro no shim" são as que continuam sem
falha programável (Messaging, RTDB), e entram nas próximas entregas da etapa 7.

Mudanças de comportamento nesta rodada (cada uma com teste que falha sem ela):
- **gms-cast**: `onCastStateChanged(int)` agora dispara em transição real de
  estado (connect/disconnect programados). Antes, `CastStateListener` era só
  guardado e nunca chamado. Mutação (dispatch removido) ⇒ teste falha.
- **firebase-rtdb**: `removeEventListener` agora deixa de entregar eventos
  (já existia no código, sem teste). Mutação (remoção do `retain`) ⇒ teste falha.

Getters novos (somente leitura): `gms_nearby_is_discovering`, `gms_nearby_connections`.

### S10 — Apêndice D: ViewPager2 (2026-11-05)

- **S10 (rodada 2 de D, branch `appendix-d-viewpager2-20261105`)**: ViewPager2
  sobe de `stub` para `partial` (`androidx-viewpager2`).
  - Shim em `crates/rd-vm/src/androidx_widgets.rs` (`Family::Pager`), com
    estado por instância: adapter e página atual. `setAdapter` zera a
    página (como o AOSP); `setCurrentItem(I)` e `(IZ)` fixam a página em
    `[0, count-1]`; sem adapter, ou com adapter vazio, a chamada é ignorada.
  - Desvio de design: `count` vem do `getItemCount()` do DEX do adapter, pelo
    mesmo caminho do RecyclerView (`m8::call_dex`). Adapter sem
    `getItemCount` no DEX conta como 0 (chamada ignorada, sem panic).
  - Testes: 6 novos em `crates/rd-vm/tests/androidx_widgets.rs` (roundtrip e
    reset, clamp, forma com `smoothScroll`, ignorado sem adapter ou com
    adapter vazio, adapter só do host, isolamento por instância), com
    dispatch real por `Engine::invoke_instance`. O arquivo tem 15 testes.
  - Mutações (cada uma derruba os testes que devem cair): sem clamp superior
    → 2 falhas; sem reset no `setAdapter` → 1 falha; sem o caso
    `count == 0` → 2 falhas. Arquivo restaurado e comparado byte a byte.
  - Inventário: a API pública completa do ViewPager2 (AndroidX main) **não**
    foi inventariada nesta rodada. O que não é modelado está listado em
    `notes` de `data/jetpack.toml`. Por isso o status é `partial`, não
    `supported`.
  - Não modelado (dito em `notes`): animação, `OnPageChangeCallback`
    (nenhum evento é entregue), fake drag, entrada de usuário, transformadores
    de página, orientação, offscreen page limit, item decorations e
    acessibilidade. Se o adapter encolher, a página não é reajustada.
  - O pacote legado `androidx.viewpager` (ViewPager v1) segue `stub` (só
    hierarquia e inflate, sem shim) e fica fora de `key_apis`.
  - D — continuam `stub` (3): `androidx-constraintlayout`,
    `androidx-coordinatorlayout`, `androidx-material`. D — `not_implemented`
    (20) sem mudança.

### S11 — Apêndice D: photo picker com resultado programável (2026-11-05)

- **S11 (rodada 3 de D, branch `appendix-d-photopicker-20261105`)**: photo
  picker sobe de `not_implemented` para `partial` (`androidx-photopicker`).
  - Escolha do usuário: resultado programável. Sem provedor inventado e sem
    sucesso fake. Por padrão, o picker cancela.
  - Lançamento: `Activity.startActivityForResult(Intent, int)` (e a forma com
    `Bundle`) em `crates/rd-vm/src/framework.rs`. Ações de picker
    (`android.media.action.PICK_IMAGES`, `GET_CONTENT`, `OPEN_DOCUMENT`) passam
    por `crates/rd-vm/src/photopicker.rs`. Outras ações continuam
    `NOT_IMPLEMENTED` e não consomem a fila.
  - Resultado: uma fila FIFO de `PickerAnswer` (`Uri` ou `Cancel`) em
    `HostState.photopicker`. Fila vazia = `RESULT_CANCELED` sem data. URI
    programada = `RESULT_OK` com a URI em `Intent.getData()`. A entrega chama
    `onActivityResult(IILandroid/content/Intent;)V` no DEX do app.
  - Modelo do Intent: único, o `IntentMeta` de `bundle.rs`. Não há arms
    duplicados de Intent. Uma tentativa anterior com arms duplicados foi
    descartada porque eles eram sombreados pelo `IntentMeta`.
  - Testes: 7 em `crates/rd-vm/tests/photopicker.rs`, com Intents criados
    por DEX (`new-instance` + `invoke-direct <init>(String)`) e lançamento por
    DEX (`invoke-virtual startActivityForResult`). Cobrem: cancelamento sem
    resposta, URI entregue com RESULT_OK e getData, FIFO com fila vazia
    cancelando de novo, GET_CONTENT e OPEN_DOCUMENT, ação fora do picker sem
    consumir a fila, Intent sem ação, e resultado sem tipo MIME.
  - Mutações (cada uma derruba os testes que devem cair): sem entrega
    (`onActivityResult` não chamado) → 5 falhas; fila ignorada (sempre
    cancela) → 4 falhas; RESULT_OK trocado por CANCELED para URI → 2 falhas;
    GET_CONTENT removido da lista → 1 falha; data não entregue → 4 falhas;
    ação não verificada (qualquer ação vira picker) → 1 falha. Arquivos
    restaurados e comparados byte a byte.
  - Desvios documentados: entrega síncrona (no mesmo call, sem loop de
    mensagens); uma URI só (`PickMultipleVisualMedia` não é devolvido); Uri
    opaca (sem esquema nem caminho); sem tipo MIME no resultado.
  - Não modelado: UI do picker do sistema, seleção real de mídia, permissão
    de acesso à mídia, e a classe `ActivityResultContracts$PickVisualMedia`
    do AndroidX como golden (os testes exercitam o caminho da ação do Intent).
  - Inventário: a API pública do photo picker não foi inventariada
    integralmente. O que não é modelado está em `notes` de `data/jetpack.toml`.
    Por isso o status é `partial`, não `supported`.
  - D — `partial` agora: DrawerLayout, SwipeRefreshLayout, CardView,
    ViewPager2, photo picker. `stub` (3): ConstraintLayout, CoordinatorLayout,
    material. `not_implemented` (19, eram 20).

### S12 — Apêndice D: RxJava 3 como dex-passthrough (2026-11-05)

- **S12 (rodada 4 de D, branch `appendix-d-rxjava-20261105`)**: RxJava sobe de
  `not_implemented` para `partial` (`rxjava`, `dex-passthrough`). O bytecode
  real da biblioteca executa na VM, sem shim host.
  - Fixture: DEX de `io.reactivex.rxjava3:rxjava` 3.1.10 e `reactive-streams` 1.0.4,
    reduzido por R8 8.7.18 com regras keep. Tem 23 KB e está em
    `crates/rd-vm/tests/fixtures/rxjava3/`. A regeneração reproduz o SHA-256
    `4023c229…` (`scripts/gen-rxjava3-fixture.sh`). Proveniência e licenças no README.
  - Exceção de política: `.dex` é bloqueante no RD Sentinel. A exceção datada, com
    escopo de um único arquivo e validade até 2027-02-05, está em
    `docs/adr/036-excecao-fixture-dex-rxjava.md`. Ela depende de um mantenedor aplicar
    `sentinel:policy-approved`. Não foi aplicada por mim.
  - Testes: 5 em `crates/rd-vm/tests/jetpack_rxjava.rs`. Cobrem `just` + `subscribe`
    → disposed; `never` → não disposed; `dispose()` → disposed; `empty` → disposed;
    e sem o DEX da biblioteca no Engine a chamada falha.
  - Gap de plataforma encontrado pelo teste: `AtomicInteger.lazySet` e
    `AtomicReference.lazySet` não existiam na VM. Os dois viram alias de `set`
    (`crates/rd-vm/src/intrinsics.rs`). Na VM single-threaded essa é a semântica.
  - Mutações (cada uma derruba os testes certos; arquivo restaurado e comparado
    byte a byte): sem `lazySet` em AtomicInteger → 1 falha; sem `lazySet` em
    AtomicReference → 2 falhas; `set`/`lazySet` de AtomicReference não grava → 2 falhas.
  - Não modelado: Schedulers e threads, Flowable e backpressure, operadores além de
    just/empty/never, blocking*, onError/RxJavaPlugins, CompositeDisposable, Subjects.
    Por isso o status é `partial`, não `supported`.
  - D — `partial` agora: DrawerLayout, SwipeRefreshLayout, CardView, ViewPager2,
    photo picker, RxJava (6). `stub` (3): ConstraintLayout, CoordinatorLayout,
    material. `not_implemented` (18).

### S13 — Apêndice D: Room 2.6.1 real sobre o VM (ST-02) (2026-11-08)

- **S13 (rodada 5 de D, branch `fix/room-migrations-close-20261108`)**: o Room
  deixa de ser um builder com migrações no-op. Três desvios do VM, achados por um
  fixture real, foram corrigidos, com aprovação do usuário para mudar a semântica
  que afeta todos os apps:
  - `addMigrations` grava as migrações no `MigrationContainer` real do Room
    (`crates/rd-vm/src/m8.rs`). Antes era no-op: a migração 1→2 nunca rodava.
  - `requireMigration` segue o padrão do Room. Sem `fallbackToDestructiveMigration()`,
    uma versão sem caminho de migração lança `IllegalStateException`
    ("A migration from 1 to 2 was required but not found"). Antes, o VM recriava
    a tabela em silêncio e perdia os dados.
  - `close()` libera a conexão do backend (`crates/rd-vm/src/jsqlite.rs`), como o
    Android. Antes ficava com a conexão viva, e a tabela TEMP do invalidation
    tracker sobrevivia; a reabertura falhava com `table room_table_modification_log
    already exists`. O `close` do `SQLiteOpenHelper` (que o Room chama ao fechar)
    também libera a conexão do banco que abriu.
  - Evidência do bug antes da correção (trace SQL, `RD_SQL_TRACE=1`): com a migração
    declarada, nenhum `ALTER TABLE` era executado; o Room rodava `DROP TABLE IF
    EXISTS item` e recriava a tabela. Sem migração, o mesmo `DROP` acontecia sem erro.
  - Fixture: `crates/rd-vm/tests/fixtures/room/src/` (duas versões do mesmo banco:
    v1 com `item(id, name)`; v2 com `qty INTEGER NOT NULL DEFAULT 0` e
    `Migration(1,2)`). O `_Impl` é gerado pelo processador real do
    `androidx.room:room-compiler:2.6.1` no javac, e o DEX sai do D8 (R8 8.7.18,
    `--min-api 26`). `scripts/gen-room-fixture.sh` regenera tudo a partir do Google
    Maven (AndroidX não existe no Maven Central), com `platform-35` para o
    `android.jar`. O SHA-256 gerado nesta rodada foi `134e9ab7…`.
  - O DEX **não** é versionado: `.dex` é bloqueante no RD Sentinel e a exceção da
    ADR 036 cobre só o fixture do RxJava. Por isso `crates/rd-vm/tests/jetpack_room.rs`
    (3 testes) é `#[ignore]` e exige `RD_ROOM_FIXTURE_DEX`. Rodar no CI exige uma
    exceção de política nova, que é decisão de mantenedor.
  - Testes: `migration_declared_runs_alter_and_keeps_rows` (`count=1,qty=0`),
    `without_migration_refuses_to_open_like_room` (ISE do Room) e
    `fresh_database_reopens_in_same_process` (reabertura após `close`).
    Os três passam com a correção. No VM original (`main`) os três falham.
  - Regressão: `cargo test -p rd-vm -- --include-ignored` com o DEX presente: todos
    os blocos `test result: ok`, zero falhas.
  - (Superado pelo S21) `fallbackToDestructiveMigrationFrom`/`OnDowngrade` não eram gravados no config.
  - Não modelado: auto-migrations, observers do InvalidationTracker
    (triggers) e coroutines/Flow. Por isso o status é `partial`, não `supported`.
  - Observado no trace: cada abertura em conexão nova reexecuta o `onOpen` do Room
    (`CREATE TEMP TABLE`), e agora ele não falha. Sem erro de SQL nos três cenários.

### S14 — Apêndice D: navigation, desempilhar a raiz como o AOSP (2026-11-08)

- **S14 (rodada 6 de D, branch `fix/navigation-root-pop-20261108`)**: `popBackStack()` e
  `popBackStack(id, inclusive)` no host de `NavController` passam a seguir o AOSP.
  - Fonte: `NavController.java` (AOSP `platform/frameworks/support`): `popBackStack()` é
    `popBackStack(atual, true)`; o retorno é `popped && dispatchOnDestinationChanged()`, e
    o dispatch não ocorre com a pilha vazia. A doc oficial de back stack diz que, quando
    o retorno é false, `getCurrentDestination()` é nulo (o último destino saiu da pilha).
  - Antes: a raiz era mantida em qualquer caso, e `popBackStack(raiz, true)` retornava 1.
    Agora desempilhar a raiz esvazia a pilha e retorna 0.
  - `navigateUp()` não muda: o AOSP também não desempilha a raiz nesse caso (retorna false).
  - Decisão do usuário (2026-11-08, "pesquise e escolha a melhor solução"): alinhar ao
    Android. A mudança afeta apps que desempilham a raiz; apps que tratam o retorno/destino
    nulo se comportam como no Android.
  - Teste: `dex_navigation_root_pop_empties_stack_like_android` (`crates/rd-vm/tests/androidx.rs`).
    Mutação (retorno `popped` sem checar pilha vazia) derruba o teste.
  - Não modelado: `navigate()` com pilha vazia (o AOSP exige o destino atual),
    `popUpTo`/`saveState`/`restoreState`, e validação de destino desconhecido ao navegar.
    Por isso o status segue `partial`.

### S15 — Apêndice D: testes do Room no CI, com DEX gerado na hora (2026-11-08)

- **Decisão do usuário (2026-11-08, `room_dex_policy`): gerar no CI.** O DEX do fixture do
  Room continua **não versionado** (ADR 036 inalterada: a exceção cobre só o RxJava).
- Novo job `room-fixture` em `.github/workflows/ci.yml`: roda `scripts/gen-room-fixture.sh`
  (Room 2.6.1 com o processador real no javac, D8/R8 8.7.18, ~24 s) e depois
  `RD_ROOM_FIXTURE_DEX=… cargo test -p rd-vm --test jetpack_room -- --ignored`.
- Validado localmente com os mesmos comandos: SHA-256 do DEX `134e9ab7…` (igual ao da S13),
  3 testes do Room passam.
- Os 3 testes continuam `#[ignore]`, para que `cargo test --workspace` não dependa do DEX.
- Status de `room` segue `partial` (ver S13 e S21: não modelados auto-migrations, observers do
  InvalidationTracker e coroutines/Flow).

### S16 — Apêndice D: lacunas conhecidas de WorkManager (2026-11-08)

Auditoria do modelo host de `androidx.work` em `crates/rd-vm/src/m8.rs` (sem mudança de
comportamento nesta rodada). Achados, por leitura do código:

- `cancelWorkById`, `cancelAllWorkByTag` e `cancelUniqueWork` percorrem `vm.fw.m8.work` e
  marcam **todo** registro `ENQUEUED` como `CANCELLED`. Não casam id, tag nem nome único.
  Efeito: cancelar um trabalho cancela os outros. Isso é um erro, não só uma lacuna.
- `WorkRecord` guarda `id` no formato `work-N`, não um UUID. Não há `getId()`, então o app
  não tem como relacionar um `WorkRequest` ao registro.
- `tags` é sempre `Vec::new()`: `addTag` não é gravado. Por isso `cancelAllWorkByTag` e
  `getWorkInfosByTag` não teriam como casar.
- `getWorkInfosByTag` e `getWorkInfosForUniqueWork` devolvem um `ListenableFuture` sem valor.

Correção (2026-11-08, branch `fix/workmanager-cancel-ids-20261108`):

- Cada `WorkRequest` ganha id UUID v4 no `build()` (RNG por engine, determinístico com
  `RD_UUID_SEED`); `WorkRequest.getId()` devolve o mesmo `java.util.UUID`, e `WorkRecord.id` usa esse texto.
- `Builder(Class)` grava o worker (antes o `worker_class` saía sempre vazio); `addTag` acumula
  as tags no builder e o `build()` as copia; `WorkRecord.tags` as recebe.
- `enqueueUniqueWork` / `enqueueUniquePeriodicWork` gravam o nome em `WorkRecord.unique_name`.
- `cancelWorkById`, `cancelAllWorkByTag` e `cancelUniqueWork` cancelam só os `ENQUEUED` que casam.
- `getWorkInfosByTag` / `getWorkInfosForUniqueWork` deixaram de ser sequestrados. Sem `WorkInfo`
  real, a chamada não devolve mais um future vazio: falha de forma explícita (não modelado).

Testes: `crates/rd-vm/tests/m8.rs` (`m8_workmanager_*`, incluindo mutação do casamento que derruba
os testes de cancelamento) e `crates/rd-vm/tests/m8_workmanager_dex.rs` (bytecode DEX em memória:
builder, `getId`, `enqueue`, `cancelWorkById`).

Segunda rodada (mesma branch de correção, 2026-11-08): `WorkInfo` real.

- `getWorkInfosByTag`, `getWorkInfosForUniqueWork` e `getWorkInfoById` devolvem `ListenableFuture`
  já completo, com `List<WorkInfo>` (ou `WorkInfo`/`null` por id). `WorkInfo.getId()` (UUID) e
  `getState()` (enum `WorkInfo.State`, constantes com a ordem da AndroidX, cache em `statics`).
  `WorkInfo.State.isFinished()`. `sget` de `WorkInfo.State.X` resolve pelo provider host
  (`m8::host_static`, em `engine.rs`).
- Teste DEX: `crates/rd-vm/tests/m8_workmanager_dex.rs` (lista, `Future.get`, `List.get`, `getState`
  comparado com `sget` por identidade, `isFinished`). Limite: o teste não pega o cache de identidade
  quebrado em referências guardadas de uma chamada anterior (mutação aplicada e não detectada).

Terceira rodada: `ExistingWorkPolicy` e `ExistingPeriodicWorkPolicy` são enums host (`host_static`).
`KEEP` descarta o pedido novo quando já há trabalho `ENQUEUED` com o mesmo nome único; `REPLACE`
cancela esse trabalho (só `ENQUEUED`, como o cancelamento) e enfileira o novo. Teste:
`dex_unique_work_keep_and_replace_follow_existing_work_policy` (mutações de KEEP e REPLACE derrubam o teste).
`APPEND`, `APPEND_OR_REPLACE` e `UPDATE` não são modeladas (encadeamento depende de execução).

Fechado em S22: tags. Fechado em S23: `doWork()` sob demanda
(`m8_run_work`) e progresso/output em `WorkInfo` (ver S23). Fechado em S24: `Constraints`
aplicadas contra o estado simulado do dispositivo (ver S24). Por isso `work` segue `partial`
(sem o runtime real do WorkManager e sem chains).

### S17 — Apêndice D: security-crypto sobre o AndroidKeyStore (ST-04) (2026-11-08)

- **S17 (rodada 7 de D, branch `feat/appd-security-crypto-20261108`)**: `EncryptedSharedPreferences.create`
  (assinaturas com `MasterKey` e com alias deprecado) e a superfície `SharedPreferences`/`Editor` sobem de
  `not_implemented` para `partial`. Módulo `crates/rd-vm/src/security_crypto.rs`, com 14 testes unitários
  (codec, cifragem, AAD por arquivo e por chave, chave mestra ausente ou diferente, tipos errados, edições
  pendentes, `clear` seguido de `put`, listeners). A API pública foi conferida no AAR oficial 1.1.0
  (`dl.google.com`), não só na documentação.
- Valores cifrados com AES-256-GCM pela chave do AndroidKeyStore (`jkeystore.rs`), IV aleatório por valor e
  AAD = arquivo + chave. Mutação de teste: remover o arquivo do AAD derruba `value_copied_to_another_file_*`.
- **Desvios** (também em `notes` do registro): o formato não é o do Tink, então não há interoperabilidade
  com dados gravados por um dispositivo real; só os valores são cifrados (nomes ficam em claro na memória
  do host); o estado dura só a sessão do Engine (nada no VFS; superado pelo S20); `create` é interceptado mesmo com o DEX da
  biblioteca presente. `MasterKey.Builder` e `MasterKeys.getOrCreate` rodam no DEX sobre o ST-04.
- **Decisão do usuário (2026-11-08)**: o escopo de D nesta rodada são os host-shim (CameraX, security-crypto,
  credentials). Os dex-passthrough continuam `not_implemented` sem teste de carga de DEX real.

### S18 — Apêndice D: credentials, host-shim sem provedor (2026-11-08)

- **S18 (rodada 8 de D, branch `feat/appd-credentials-20261108`)**: `CredentialManager` sobe de
  `not_implemented` para `partial`. Módulo `crates/rd-vm/src/credentials.rs`, com 6 testes de guest em
  `crates/rd-vm/tests/credentials_shim.rs`: o DEX sintético chama a interface por `invoke-interface/range`
  e o callback é uma classe do próprio DEX que guarda o que recebe. A API foi conferida no AAR oficial
  androidx.credentials 1.7.0-alpha03 (`dl.google.com`, `javap`).
- **Padrão sem provedor** (sem provedor fake instalado; o fake é opt-in, ver S18b): `getCredentialAsync` → `onError(NoCredentialException)`;
  `createCredentialAsync` → `onError(CreateCredentialUnsupportedException)`; `clearCredentialStateAsync` →
  `onResult(null)`, porque não há estado a limpar. Os campos `type` e `errorMessage` seguem o construtor
  real (constantes lidas do bytecode). Argumentos obrigatórios nulos (contexto, request, callback) → NPE.
- Mutação: desligar o dispatch de instância derruba 5 dos 6 testes.
- **Desvios** (também em `notes` do registro): callback entregue inline, antes de a chamada retornar; executor
  ignorado; o construtor do DEX não roda (o host grava os campos); nada é persistido, porque sem provedor não
  existe credencial.
- **Não modelados** (caem no DEX da biblioteca): `prepareGetCredentialAsync`,
  `getCredentialAsync(PendingGetCredentialHandle, …)`, `createSettingsPendingIntent`, `signalCredentialState*`
  e as variantes `suspend`.
- **Pendente (resolvido em S18b)**: o caminho de sucesso só ficava testável com um provedor fake.

### S18b — Apêndice D: provedor fake do credentials (2026-11-08)

- **Pesquisa**: o Credential Manager é atendido por provedores do sistema (`CredentialProviderService`, Android 14+
  para terceiros). O app não instala provedor no próprio processo, então um "provedor fake" é dublê de teste.
  A documentação oficial pede tratar `NoCredentialException`, cancelamento, interrupção, erro de configuração
  e falha desconhecida (`developer.android.com/identity/sign-in/smart-lock-migration`).
- **Opções avaliadas**: (1) manter o padrão sem provedor, sem caminho de sucesso testável; (2) provedor fake
  programável, opt-in, só para `get`, no formato dos `gms_*_set_*` do Apêndice E; (3) provedor completo com
  armazenamento, leitura das opções do `GetCredentialRequest` e `preferImmediatelyAvailableCredentials`.
- **Escolhida: (2).** Cobre o caminho de sucesso e as falhas tipadas sem a complexidade da (3), que depende de
  ler as opções do request e de um armazenamento para o create. A (3) fica como próximo passo.
- **Implementação**: `credentials_set_password(id, senha)` faz `getCredentialAsync` devolver
  `GetCredentialResponse` com um `PasswordCredential` (campos `type`, `id`, `password` e a lista `credentials`,
  como o construtor real grava). `credentials_set_get_failure(GetFailure)` faz o get falhar com
  `GetCredentialCancellationException`, `GetCredentialInterruptedException`,
  `GetCredentialProviderConfigurationException` ou `GetCredentialUnknownException`, com o `type` lido do bytecode
  do AAR 1.7.0-alpha03. `credentials_clear_fake()` volta ao padrão. Estado em `HostState.credentials`, padrão `None`.
- **Desvios**: o fake ignora o `GetCredentialRequest` e responde igual a qualquer pedido. `PasswordCredential.data`
  fica nulo (o `Bundle` não é modelado). Create e clear não mudam com o fake.
- **Testes**: 4 novos em `credentials_shim.rs`: sucesso com `PasswordCredential` (id, senha e type conferidos);
  cada uma das 4 exceções `Get*` com a classe e o `type` da biblioteca; ciclo sucesso → erro → limpar → volta ao
  padrão sem resíduo; create e clear inalterados. Mutação do dispatch (ignorar o fake): derruba 3 dos 4.
- **Próximo passo (não feito)**: (3), com armazenamento para o create e leitura das opções do request.

### S19 — Apêndice D: CameraX, validação ponta a ponta do DEX real (2026-11-08)

- **Decisão do usuário (2026-11-08)**: primeiro, escrever um shim host do ponto de entrada do CameraX. Depois
  da evidência abaixo, o usuário escolheu validar o DEX real. Esta rodada não tem shim novo; o plano
  "host-shim" para camerax fica substituído por esta decisão.
- **Evidência (local, build `rd-cli` debug de `main` 9399bf3, APK `comprix-v1.6.0-x86_64-release.apk`)**:
  - Rota até o Scanner: `Pular` → `Nova lista` → `Compra do Mês` → `Criar e adicionar itens` →
    `Escanear um produto`, com permissão de câmera concedida. `last_error` = `null`. A tela "Escanear preço"
    mostra o preview sem injeção de frames (`/home/user/comprix/cx_run1.png`, fora do repositório).
  - Trace com `RD_TRACE_METHOD=Landroidx/camera`: `ProcessCameraProvider.getInstance` →
    `getOrCreateCameraXInstance`, criação do `PreviewView`, binding de use cases, `SurfaceRequest` e
    `AndroidImageReaderProxy`. Todo o pipeline interno do CameraX roda no DEX sobre a Camera2 virtual (`m10`).
  - "Tirar foto": `ImageCapture.takePicture` → `submitStillCaptureRequest` executa, com `FutureChain` e
    `Futures.transformAsync`. **A conclusão da captura não aparece no trace** (sem `onCaptureSuccess` nem
    `ImageSaver`), então o sucesso da captura não é afirmado.
  - **Reteste (2026-11-08, 22:15, `rd-cli` de `main` 8c3e9f2)**: com `RD_TRACE_METHOD=Lbr/com/comprix/`, que
    cobre os callbacks do app, nenhum `onCaptureSuccess`, `onError` ou `onImageSaved` aparece. O screenshot
    pós-"Tirar foto" (`/home/user/comprix/cx_capture.png`, fora do repositório) ainda mostra a tela
    "Escanear preço", sem a tela de confirmação do que o app leu. `last_error` = `null`. A captura continua
    não confirmada.
- **Decisão de status (2026-11-08, usuário)**: o usuário aceitou a evidência local e pediu que o CameraX vire
  `partial`. Registro: `module = "rd-vm/m10"`, `code` = `m10/camera.rs` e `m10/preview.rs`, `gap_id` mantido.
  Os testes no repositório cobrem a camada Camera2 que o CameraX usa: `camera_305.rs`, `m10.rs` e
  `android_scale_gesture.rs` (caminho do `PreviewView`). A prova do DEX do CameraX em si continua local, porque
  o APK não é versionável. Isso abre uma exceção à regra de `partial` com teste específico, registrada aqui e
  no PR. Não houve fixture com ADR. A estratégia na tabela segue `host-shim`, como o escopo de D, mas a
  implementação é o DEX real.
  Sobre o registro anterior ("pendente: fixture ou aceitar evidência local"): fica resolvido pela aceitação acima.
- **Limites da evidência**: o preview foi verificado visualmente (screenshot). Não foi medido frame a frame, e a
  captura não foi confirmada. `ImageAnalysis` (modo Código) não foi exercitado.

### S20 — Apêndice D: security-crypto, persistência entre sessões (2026-11-08)

- **Lacuna fechada**: até o S19, o estado do security-crypto durava só a sessão do Engine. O cabeçalho de
  `security_crypto.rs` dizia "nada é gravado no VFS", e isso não é mais verdade. O VFS da sessão
  (`Engine::set_vfs`) guarda agora duas coisas:
  - o arquivo cifrado em `files/rd_encrypted/<nome em hex>.enc`, em formato de texto (`RDENC1`, uma linha por
    valor com IV e `ciphertext||tag` em hex), regravado a cada `commit`/`apply` e na criação;
  - a chave do AndroidKeyStore em `files/rd_keystore/<alias em hex>.key`, gravada em `generateKey` e apagada
    em `deleteEntry`.
- **Recarga**: `create` recarrega a chave e o arquivo do VFS quando a Engine não os tem em memória.
  `containsAlias`, `getKey`, `Cipher.init` e `doFinal` também recarregam a chave. A leitura de valor não
  recarrega: uma chave removida da memória continua falhando fechado.
- **Falhas tipadas**: arquivo presente mas inválido → `GeneralSecurityException` (não vira arquivo vazio, para
  não sobrescrever dados); chave removida → `GeneralSecurityException` no `create`; byte alterado no
  ciphertext gravado → `SecurityException` na leitura da sessão seguinte.
- **Desvios** (também em `notes` do registro): o material da chave fica em claro no VFS, sem TEE/StrongBox
  (o AndroidKeyStore real não expõe o material); o formato é próprio, não o do Tink; os nomes de arquivo e de
  chave aparecem em hex no VFS (sem separador de caminho), não cifrados.
- **Testes** (`crates/rd-vm/src/security_crypto.rs`, 4 novos; módulo com 18 testes):
  - `values_and_keystore_key_survive_a_new_engine_on_the_same_vfs`: a Engine B, sem nada em memória, lê o valor
    gravado pela Engine A; o texto puro não aparece no arquivo gravado;
  - `deleted_keystore_key_is_not_loaded_by_the_next_engine`;
  - `tampered_file_on_the_vfs_fails_authentication_in_the_next_engine`;
  - `corrupted_file_on_the_vfs_is_a_typed_error_not_an_empty_file`.
- **Mutação**: sem a gravação no `apply`, caem os 2 testes de sessão que dependem dela; sem a recarga da chave
  no `create`, caem os mesmos 2. O arquivo foi restaurado e conferido por checksum.
- **Validação**: `cargo test -p rd-vm` completo passa, com 0 falhas; `cargo clippy -p rd-vm --lib --tests -- -D warnings`
  ok; `cargo fmt --all -- --check` ok.
- **Fora deste passo**: DataStore (fica para depois, depende de coroutines e de persistência em disco);
  AES256_SIV para nomes de chave; EncryptedFile; o caminho Tink do DEX.

### S21 — Apêndice D: Room, fallbackToDestructiveMigrationFrom e OnDowngrade (2026-11-08)

- **Lacuna fechada**: o builder aceitava `fallbackToDestructiveMigrationFrom(int...)` e
  `fallbackToDestructiveMigrationOnDowngrade()` sem gravar nada. O config recebia `migrationNotRequiredFrom`
  vazio e `allowDestructiveMigrationOnDowngrade` no padrão, então o Room recusava aberturas que o app real
  aceitaria. Isso era mais estrito que o Room, nunca mais permissivo (S13).
- **Implementação** (`crates/rd-vm/src/m8.rs`): o builder grava as versões num `HashSet<Integer>` (boxed) e o
  flag de downgrade. `room_init` passa o conjunto para `migrationNotRequiredFrom` e o flag para
  `allowDestructiveMigrationOnDowngrade`. A decisão de `isMigrationRequired` continua no Room real (DEX), não no host.
- **API conferida no jar** (`javap` em `room-runtime-2.6.1.jar`): `fallbackToDestructiveMigrationFrom(int...)`
  (só a forma varargs) e `fallbackToDestructiveMigrationOnDowngrade()`. A bytecode de `isMigrationRequired`
  confirma que o conjunto e o flag são os campos consultados.
- **Fixture**: `V2.java` ganha `seed` e `withFallbackFrom`; `V1.java` ganha `openOverV2`,
  `openOverV2WithoutFallback` e `count` no DAO. O DEX foi regenerado por `scripts/gen-room-fixture.sh`
  (javac com o processador do Room 2.6.1, depois D8), sem erro.
- **Testes** (`crates/rd-vm/tests/jetpack_room.rs`, `#[ignore]`; a CI roda com o DEX):
  - `fallback_from_listed_version_recreates_the_table`: `count=0`;
  - `downgrade_with_fallback_recreates_the_table`: `count=0`;
  - `downgrade_without_fallback_refuses_to_open_like_room`: `IllegalStateException`
    ("A migration from 2 to 1 was required but not found");
  - os 3 testes anteriores seguem passando. Total: 6/6 com `--ignored`.
- **Mutação**: conjunto sempre vazio derruba `fallback_from_listed_version_recreates_the_table`; flag de downgrade
  fixo em 0 derruba `downgrade_with_fallback_recreates_the_table`. O arquivo foi restaurado e conferido por checksum.
- **Limite**: o fixture só tem os pares 1→2 e 2→1. Versões de origem diferentes de 1 não têm cenário próprio.
  Não há auto-migrations, observers do InvalidationTracker, nem coroutines/Flow. Status segue `partial`.

### S22 — Apêndice D: WorkInfo.getTags (WorkManager, 2026-11-08)

- **Lacuna fechada** (pendência do S16: "tags/progresso/output em `WorkInfo`"): só as tags. `WorkInfo`
  agora devolve `getTags()` com um `Set<String>` das tags do pedido, como o real. Antes, `WorkInfo` só tinha
  `getId()` e `getState()`.
- **Implementação** (`crates/rd-vm/src/m8.rs`): `new_work_info` monta `rd_tags` a partir do `WorkRecord.tags`
  (que já vinha do `addTag` desde o S16), e o despacho de `WorkInfo.getTags` lê esse campo.
- **Teste** (`crates/rd-vm/tests/m8.rs`, `m8_workmanager_work_info_exposes_tags`): enfileira um pedido com
  `sync` e `daily`, pega o `WorkInfo` por `getWorkInfosByTag("sync")` (o future já completo) e confere
  `getTags()` = {`daily`, `sync`}. Os 8 testes de `workmanager` passam.
- **Mutação**: `WorkInfo` com o conjunto de tags vazio derruba o teste (`left == right`). Arquivo restaurado e
  conferido por checksum.
- **Continua pendente**: progresso e output em `WorkInfo`, `Worker.doWork()` em background, e a aplicação de
  constraints. Por isso `work` segue `partial`.

### S23 — Apêndice D: doWork sob demanda, progresso e output (WorkManager, 2026-11-09)

- **Lacuna fechada** (pendência do S22): `Worker.doWork()` agora roda. O harness
  `Engine::m8_run_work(id)` cria o Worker do DEX com o construtor real
  `(Context, WorkerParameters)`, chama `doWork()` e mapeia o `Result`:
  `success` → `SUCCEEDED`, `failure` → `FAILED`, `retry` → volta a `ENQUEUED`, `null` → `FAILED`.
  Exceção não capturada no construtor ou em `doWork` vira `FAILED`, como no WorkManager. Erro interno
  da VM propaga e o pedido volta a `ENQUEUED`.
- **Progresso e output**: `Worker.setProgressAsync(Data)` grava o progresso no pedido; `Result.success(Data)`
  e `failure(Data)` gravam o output. `WorkInfo.getProgress()` e `getOutputData()` devolvem esses `Data`;
  sem valor, um `Data` vazio (como `Data.EMPTY`).
- **Modelo host**: a superclasse `Landroidx/work/Worker;` é host (como `RecyclerView.Adapter`). Métodos do
  app que o DEX define mandam; os da base (`getInputData`, `getId`, `getTags`, `getApplicationContext`,
  `getRunAttemptCount`, `isStopped`, `setProgressAsync`) são respondidos pelo host. `WorkerParameters` é um
  objeto host com id, input vazio e tags. `getApplicationContext()` devolve `null` (sem contexto do app).
- **GC**: `Data` de progresso e de output, e o worker em execução, são raízes fortes.
- **Testes** (`crates/rd-vm/tests/m8_workmanager_run.rs`, 9 testes): success, failure, retry (reagendado
  e executável de novo), `null`, só roda o que está ENQUEUED, progresso e output, getters de `WorkInfo`
  pelo despacho real, `WorkInfo` vazio antes de rodar, e sobrevivência a `gc()`.
- **Mutações**: (A) `setProgressAsync` sem gravar e (B) `retry()` virando `FAILED` derrubam 4 testes
  (progresso, output, getters e retry). Arquivo restaurado e conferido por checksum.
- **Continua pendente**: `Constraints` (não aplicadas), e o runtime real do WorkManager (`WorkerParameters`
  e `Context` são modelos host). Quando o APK traz o `work-runtime` real, o DEX real manda nos métodos que ele define.

### S24 — Apêndice D: Constraints do WorkManager (2026-11-09)

- **Lacuna fechada**: as constraints deixam de ser só aceitas. `Constraints.Builder` é host
  (`setRequiredNetworkType(NetworkType)`, `setRequiresCharging`, `setRequiresBatteryNotLow`,
  `setRequiresStorageNotLow`, `setRequiresDeviceIdle`, `build()`), e `WorkRequest.Builder.setConstraints(c)`
  copia as constraints para o pedido. `NetworkType` é enum host (`NOT_REQUIRED`, `CONNECTED`, `UNMETERED`,
  `NOT_ROAMING`, `METERED`, `TEMPORARILY_UNMETERED`).
- **Aplicação**: `m8_run_work` consulta `constraints_met(constraints, DeviceConditions)`. Não atendida: o
  pedido **fica ENQUEUED e o doWork não roda** (como no WorkManager).
- **Estado do dispositivo simulado**: `Engine::m8_device` / `m8_set_device`. Padrão: rede Wi-Fi (`UNMETERED`),
  fora da tomada, bateria e armazenamento OK, não ocioso. O harness muda esse estado.
- **Aproximações** (documentadas): `NOT_ROAMING` vale como `CONNECTED`; `TEMPORARILY_UNMETERED` como
  `UNMETERED`. Não há modelo de roaming nem de cota temporária.
- **Testes** (`crates/rd-vm/tests/m8_workmanager_run.rs`, 15 no total): constraints chegam ao pedido;
  sem constraints roda offline; `setRequiresCharging` bloqueia até carregar (sem progresso enquanto bloqueado);
  `CONNECTED` bloqueia offline e aceita rede medida; `UNMETERED` recusa rede medida; ocioso espera o
  dispositivo ficar ocioso.
- **Mutação**: desligar a checagem em `m8_run_work` derruba 4 testes de bloqueio. Arquivo restaurado e
  conferido por checksum.
- **Continua pendente**: o runtime real do WorkManager (`WorkerParameters`/`Context` são modelos host),
  chains (`beginWith`/`then`) e `PeriodicWorkRequest` com constraints. Por isso `work` segue `partial`.

### S25 — Apêndice D: navegação com ações e argumentos (safe-args, UI-04, 2026-11-09)

- **Lacuna fechada** (fallback do `NavController`): ações do grafo. `Engine::androidx_nav_declare_action`
  declara `<action id, from, to>`. `navigate(actionId)` e `navigate(actionId, Bundle)` saem do destino
  atual: ação declarada a partir dele leva ao destino `to`; ação declarada de outro destino lança
  `IllegalArgumentException` e não muda a pilha, como no AOSP.
- **Safe-args**: `navigate(NavDirections)` lê `getActionId()` e `getArguments()` do objeto gerado pelo
  app (método DEX real). Ação fora do grafo lança `IllegalArgumentException`.
- **Argumentos**: o `Bundle` fica na entrada empilhada (`args` paralelo à pilha) e volta por
  `NavBackStackEntry.getArguments()`. As referências são raízes do GC (`gc_host_ephemerons`).
- **Testes** (`crates/rd-vm/tests/androidx.rs`, +2): resolução de ação a partir do destino atual com
  argumentos e GC; `NavDirections` com DEX real e ação não declarada.
- **Mutação**: remover a checagem de origem da ação derruba o teste de ação; não guardar o Bundle derruba
  o teste de argumentos; remover a raiz do GC derruba o teste de sobrevivência. Arquivos restaurados
  e conferidos por checksum.
- **Continua pendente** (por isso `navigation` segue `partial`): o grafo é declarado pelo harness e não
  lido de `res/navigation/*.xml` compilado no APK; não há `NavOptions`/`popUpTo`; o fallback de
  Fragment (back stack e lifecycle) continua do DEX real. UI-04 segue aberto no roadmap.

## Critério de erro e ciclo de vida (Apêndice E, etapa 6)

Critério por serviço parcial: além do caminho de sucesso, o shim tem um caminho de erro programável
e um teste do ciclo completo (sucesso → erro → recuperação, sem resíduo do erro).

- **Play Integrity** (`gms-integrity`): `gms_set_integrity_failure(code)` faz `requestIntegrityToken`
  falhar com `IntegrityServiceException` (códigos oficiais: -1 API não disponível, -3 rede, -100 erro interno).
  O pedido falho não registra nonce. Testes em `crates/rd-vm/tests/gms_erro_ciclo.rs`.
- **Fit** (`gms-fit-health`): `gms_fit_set_failure(code)` faz leitura, escrita, gravação e sessões falharem
  com `ApiException` sem efeito colateral (sem gravação, sem sessão iniciada). Testes no mesmo arquivo.
- Mutações da checagem de falha em cada shim derrubam os testes correspondentes.

- **Fused Location** (`gms-location`): `gms_location_set_failure(code)` faz leitura, disponibilidade e
  registro de atualizações falharem com `ApiException` sem registrar callback; remover continua disponível.
  Testes no mesmo arquivo, com callback DEX que prova a entrega do convidado.

- **Remote Config** (`firebase-remoteconfig`): `gms_remoteconfig_set_fetch_failure(code)` faz `fetch` e
  `fetchAndActivate` falharem com `FirebaseRemoteConfigFetchException`; os valores ativados continuam valendo
  e nenhum valor fica pendente para o `activate`. Testes no mesmo arquivo.

- **Cast** (`gms-cast`): `gms_cast_set_connect_failure(code)` faz a tentativa falhar sem criar sessão (estado
  NO_DEVICES); o erro fica em `gms_cast_last_error` e a tentativa seguinte bem-sucedida o limpa. Testes no mesmo arquivo.
- **Nearby** (`gms-nearby`): critério já atendido por testes existentes em `tests/gms.rs` (ENDPOINT_UNKNOWN para
  endpoint não descoberto; ciclo de descoberta, conexão e encerramento). Nenhuma mudança de código.
- **Analytics** (`firebase-analytics`): nome de evento (até 40), parâmetro (até 40) e propriedade de usuário (até 24)
  inválidos pela regra documentada do Firebase são rejeitados e contados em `gms_analytics_rejected`. Parâmetro
  inválido rejeita o evento inteiro (escolha de modelagem; a doc não especifica). Coleção desligada descarta sem
  contar como rejeição. Testes em `gms_erro_ciclo.rs`; mutação do validador derruba o teste.
- **Crashlytics** (`firebase-crashlytics`): no máximo 64 pares chave/valor (novas chaves além do limite são ignoradas;
  regravar chave existente continua valendo, em `setCustomKey` e `setCustomKeys`), chave e valor truncados em 1024
  caracteres (contagem por `char`). Teste em `gms_erro_ciclo.rs`; mutação do limite derruba o teste (70 em vez de 64).
- **Maps** (`gms-maps`): falha de carregamento programável (`gms_maps_set_load_failure(code)`): `MapsInitializer.initialize`
  devolve o código e `getMapAsync` não entrega o `GoogleMap` enquanto a falha durar; `gms_maps_clear_load_failure()`
  recupera. Opt-in: sem falha programada, o comportamento é o de antes. Não acoplado ao `availability_code` global,
  para não mudar o comportamento de apps com mapas. Teste em `gms_erro_ciclo.rs`; mutação do bloqueio derruba o teste.
- **Messaging** (`firebase-messaging`): falha programável (`gms_fcm_set_failure`/`gms_fcm_clear_failure`): `getToken`
  e `subscribeToTopic`/`unsubscribeFromTopic` falham com `java.io.IOException` (SERVICE_NOT_AVAILABLE), sem efeito
  colateral; `deleteToken` segue disponível. A exceção segue relatos do SDK (firebase-android-sdk issues #4053 e #5826),
  não a referência oficial. Teste em `gms_erro_ciclo.rs`; mutação do guard derruba o teste.
- **RTDB** (`firebase-rtdb`): negação de acesso programável (`gms_rtdb_set_permission_denied(true|false)`). Leituras por
  listener chamam `onCancelled(DatabaseError)` e o listener não fica registrado (o cancelamento é terminal); escritas
  não acontecem e o `CompletionListener` recebe o erro; `removeValue`, `updateChildren` e `get` falham com
  `DatabaseException`. A doc oficial diz que, com negação das regras, o callback de cancelamento é disparado com
  `PERMISSION_DENIED`. Limitação: o código numérico de `DatabaseError` não é modelado. Teste em `gms_erro_ciclo.rs`;
  mutação do setter derruba o teste.
- **SafetyNet** (dentro de `gms-integrity`): `gms_safetynet_set_failure(code)` faz `attest` falhar com `ApiException`
  sem JWS e sem registrar nonce; `gms_safetynet_clear_failure()` recupera. Teste `safetynet_failure_blocks_attestation_then_recovers`;
  mutação do guard derruba o teste. A doc oficial pede nonce com pelo menos 16 bytes; essa regra não virou falha.

Firebase Auth já cumpre o critério por testes existentes (`InvalidCredentials` como erro; `signOut` e listener de
estado como ciclo). Google Ads e Google Sign-In também já cumprem o critério por testes existentes
(`ads_banner_success_failure_and_rewarded`, `google_signin_silent_requires_host_completion`,
`google_signin_sign_out_ends_silent_session_and_sign_in_restores_it`). Com isso, a etapa 7 (os 20 serviços
`partial`) está fechada pela tabela de erro/ciclo. Registro da etapa 7 no plano: PR #607.
