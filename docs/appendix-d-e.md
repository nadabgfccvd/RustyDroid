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

  - **E — testes de erro e ciclo de vida (7 novos em
    `crates/rd-vm/tests/gms.rs`, suíte gms com 40 testes passando)**:
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
  - **E — auditoria ainda aberta**: ainda sem teste explícito de erro ou
    ciclo de vida: `firebase-analytics` (eventos e propriedades),
    `cloud-firestore` (documento ausente em `get`) e `gms-cast`
    (listeners de estado: o shim guarda o listener, mas não há estado
    observável pelo teste). Esses itens continuam `partial` com os mesmos
    `gap_id`.
  - **E — ML Kit**: continua `not_implemented`, sem dispatch.
  - **E — status**: os 20 serviços continuam `partial` com os mesmos
    `gap_id`; os `notes` em `data/gms.toml` já descrevem o comportamento
    coberto, então o registro não muda nesta rodada.
