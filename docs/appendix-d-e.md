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
    | camerax | host-shim | Depende de MEDIA-03 (câmera virtual) |
    | security-crypto | host-shim | Depende de AndroidKeyStore (ST-04) |
    | credentials | host-shim | Depende de provedor fake de credencial (decisão de produto) |
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
um teste. Testes em `crates/rd-vm/tests/gms.rs` (45 passando).

| Serviço | Sucesso | Erro | Ciclo de vida |
|---|---|---|---|
| gms-common | `availability_programmable_codes` | mesmo teste (códigos programados) | troca de código no mesmo teste |
| gms-tasks | `tasks_completion_source_roundtrip_with_dex_listener` | `tasks_failure_surfaces_exception` | `tasks_when_all_success_failure_and_complete` |
| gms-location | `location_fix_and_callback_delivery` | sem fix ⇒ `null` (mesmo teste) | `location_fix_cleared_mid_session_returns_null_until_reseeded` |
| gms-maps | `maps_ready_camera_and_markers` | **sem caminho de erro no shim** | `maps_marker_remove_drops_only_that_marker` |
| gms-billing | `billing_product_details_modern_path` | `billing_purchase_flow_and_cancel` (cancel) | `billing_listener_relaunching_flow_is_bounded` |
| gms-auth | `google_signin_silent_requires_host_completion` | ApiException 4 (mesmo teste) | `google_signin_sign_out_ends_silent_session_and_sign_in_restores_it` |
| gms-ads | `ads_banner_success_failure_and_rewarded` | `gms_ads_set_failure` (mesmo teste) | `clear_failure` no mesmo teste |
| gms-fit-health | `fit_steps_programmable` | `fit_zero_steps_yields_empty_data_set_not_fake_points` | **sem ciclo de vida no shim** (leitura sem estado) |
| gms-cast | `cast_states_follow_programmed_session` | estado NO_DEVICES sem sessão (mesmo teste) | `cast_state_listener_fires_only_on_real_transitions` |
| gms-nearby | `nearby_discovery_reports_programmed_endpoint` | `nearby_request_connection_only_reaches_discovered_endpoints` | `nearby_lifecycle_discovery_and_connections` |
| gms-integrity | `integrity_and_safetynet_deterministic_tokens` | **sem caminho de erro no shim** | `safetynet_programmed_jws_and_nonce_record_track_api_key` |
| firebase-core | `firebase_autoinit_and_strict_mode` (auto-init) | strict sem initializeApp (mesmo teste) | mesmo teste (init → uso) |
| firebase-analytics | `firebase_autoinit_and_strict_mode` (evento) | **eventos não falham no shim** | collection off + `analytics_null_user_property_value_removes_it` |
| firebase-auth | `firebase_auth_full_flow` | senha errada (mesmo teste) | `firebase_auth_state_listener_fires_on_add_and_signin` |
| firebase-messaging | `messaging_token_topics_and_delivery` | **sem caminho de erro no shim** | tópicos (mesmo teste) |
| firebase-crashlytics | `crashlytics_records_and_uncaught_bridge` | crash não capturado vira registro (mesmo teste) | `gms_logs_are_capped_with_dropped_counter` |
| firebase-firestore | `firestore_set_get_query_and_seed` | `firestore_missing_document_reads_as_absent_not_error` | `firestore_query_filter_and_snapshot_listener` |
| firebase-rtdb | `rtdb_set_resolve_and_listener` | **sem caminho de erro no shim** | `rtdb_remove_event_listener_stops_callbacks` |
| firebase-remoteconfig | `remoteconfig_fetch_activate_semantics` | **sem falha programável no shim** | fetch → activate (mesmo teste) |
| firebase-storage | `storage_seed_download_and_missing` | `storage_getbytes_size_limit_error_and_delete_lifecycle` | mesmo teste (delete) |

Os itens "sem caminho de erro no shim" não são lacunas de teste: o shim
não tem uma falha programável para exercitar. Criar um caminho de falha novo é
mudança de comportamento e fica fora desta rodada.

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
