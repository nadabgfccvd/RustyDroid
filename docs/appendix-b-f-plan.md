# Apêndices B–F — plano de implementação e estado verificado

Este documento é o plano executável dos Apêndices B, C, D, E e F de `RustyDroid.md`.
A spec é um rascunho: a fonte de verdade do estado é o código, os registros em
`data/` e os testes. Cada afirmação de "feito" aponta para um teste ou arquivo.
Nada aqui promove um status em `data/*.toml`; isso só acontece no PR que muda o código.

Estado de referência: `origin/main` após o PR #541 (`1130a44`), mais o PR desta rodada.

## 1. Critério de conclusão por apêndice

| Apêndice | Fonte executável | Critério de "completo" (o que precisa ser verdade) |
|---|---|---|
| **B** Permissões | `data/permissions.toml`, `rd-framework::fd_permissions`, `rd-vm::perms`, `rd-agent::mcp` | Toda permissão da tabela tem estado correto por nível/flag/alvo; toda flag listada na B1 tem efeito verificado por teste **ou** está registrada como bloqueio com fonte; fluxos runtime, negativas, justificativas, auto-revoke e roles têm chamador e teste. |
| **C** Superfície `android.*` | `data/api-coverage.toml`, `docs/appendix-c-literal-completion.md` | "Literal": cada classe/método/campo público de API 26–36 de cada prefixo registrado está implementado e testado **ou** listado como bloqueio. Qualquer item não auditado conta como bloqueio. |
| **D** AndroidX/Jetpack | `data/jetpack.toml`, `docs/appendix-d-e.md` | Cada biblioteca da tabela tem estratégia (`dex-passthrough`, `host-shim`, `host-native`) e status honesto; `supported` só para o subconjunto documentado com teste. |
| **E** GMS/Firebase | `data/gms.toml`, `docs/appendix-d-e.md` | Modo `no-gms-honest` validado; cada serviço tem shim programável com teste; nenhum serviço finge backend real. |
| **F** Componentes & manifest | `rd-apk::manifest` | Os 4 componentes e as extensões da tabela F são modelados; cada elemento de manifest listado tem parse e comportamento; cada extensão tem action string **verificado**. |

## 2. Linha de base verificada (contagens do código, não da spec)

Os números do texto da spec estão defasados. Valores medidos neste commit:

| Registro | Total | Estado |
|---|---|---|
| `api-coverage.toml` (C) | 106 superfícies, 17 domínios | 4 supported · 102 partial · 0 stub · 0 not_implemented |
| `jetpack.toml` (D) | 46 bibliotecas, 6 categorias | 19 partial · 7 stub · 20 not_implemented |
| `gms.toml` (E) | 21 serviços, 3 categorias | 20 partial · 1 not_implemented (ML Kit, extra) |
| `permissions.toml` (B) | 195 permissões, 13 roles | 5 com `hardRestricted`; 5 com `role = "SMS"` (após este PR) |
| Catálogo MCP | 52 tools | inclui `permissions_auto_revoke` e `permissions_role` (Apêndice B6) |

Não é possível contar o inventário literal de C: o sandbox não tem `android.jar`
(`find / -name android.jar` vazio). Sem o SDK, o total de classes públicas de API 26–36
não é mensurável aqui. Isso é um bloqueio de C, não uma estimativa.

## 3. Apêndice B — estado

### Implementado (com teste)

| Item da spec (B1–B6) | Implementação | Teste |
|---|---|---|
| Níveis normal/dangerous/signature/internal | `initial_state` | `install_classifies_by_level` |
| Flag `runtime` (diálogo) | `RuntimePending`, `grant`/`deny` | `grant_deny_revoke_lifecycle` |
| Flag `special` / `appop` | `SpecialPending` | `table_matches_documented_counts` |
| Flag `development` / `systemApp` (BATTERY_STATS) | `DeniedSystem` fora da whitelist b5 | `battery_stats_is_denied_system_outside_b5_whitelist` |
| B5 whitelist de signature | `is_b5` | `signature_entries_are_partitioned_by_b5_whitelist` |
| Gating por targetSdk / maxSdk / since_api | `NotRequested` | `max_sdk_version_gates_not_requested`, `since_api_gates_not_requested` |
| Negativas, justificativa, "não perguntar de novo" | `denials`, `shows_rationale`, `is_permanently_denied` | `first_denial_shows_rationale_second_is_dont_ask_again` |
| Justificativa no VM e no MCP | `rationale_names` → `permission_rationale` | `rationale_names_lists_only_single_denials` (unitário); ver lacuna abaixo |
| Auto-revoke por app sem uso (B6) | `auto_revoke_unused` + tool `permissions_auto_revoke` | `auto_revoke_only_for_target_30_plus_and_only_after_threshold`; `permissions_auto_revoke_follows_target_sdk_and_threshold` (MCP) |
| Acesso parcial a mídia API 34+ | `grant_partial_media` | `partial_media_grants_visual_selection_only_on_api_34_plus` |
| Flag `hardRestricted` (SMS/MMS) | `grant` recusado sem allowlist ou role | `hard_restricted_sms_is_refused_until_installer_allowlists_it` |
| Flag `immutable` | `is_frozen` em grant/deny/revoke/auto-revoke | `immutable_permission_ignores_grant_deny_and_revoke` |
| Roles (B6) — `role` | `set_role`, `roles_held`, `permissions_role` (MCP) | `role_holder_gets_role_permissions_and_loses_them_on_exit`; `role_holding_satisfies_hard_restriction_without_allowlist`; `role_grant_takes_precedence_over_an_earlier_denial`; `unknown_role_is_rejected`; `permissions_role_grants_sms_permissions_while_held` |

### Bloqueios (sem fonte verificada; não implementados)

| Flag | Evidência encontrada | Por que não implementar agora |
|---|---|---|
| `instant` | `PROTECTION_FLAG_INSTANT` existe na API oficial (`PermissionInfo`) | Exige modelo de app instantâneo, que o runtime não tem. Hoje o dado não marca nenhuma permissão. |
| `revokeWhenRequested` | `FLAG_PERMISSION_REVOKE_WHEN_REQUESTED` existe no AOSP (`PackageManagerShellCommand`, `PermissionManagerService`) | A flag existe, mas não encontrei documentação oficial de **quando** a revogação ocorre. Implementar seria chute. |
| `softRestriction` | Só `ACCESS_BACKGROUND_LOCATION` a usa; `FLAG_SOFT_RESTRICTED` existe na API | Metadado sem efeito verificado no runtime; a restrição do Android é aplicada pelo sistema. Precisa de fonte sobre o efeito antes de modelar. |

A flag `role` foi implementada: a documentação AOSP de roles diz que o role concede as
permissões declaradas em `<permissions>` ao holder. A ligação SMS → permissões de
SMS vem da política do Play (grupo SMS exige default SMS handler). A regra "holder do
role dispensa `hardRestricted`" é uma **decisão de modelagem** deste projeto, não um
fato da documentação do Android; está marcada no código.

### Lacuna conhecida de teste

A resposta de `shouldShowRequestPermissionRationale` no VM não tem teste de ponta.
Tentativa com `m10::call_instance` em receptor `Context` retornou `None` (a intrínseca
não casou o receptor). Não foi removida a lógica: segue coberta pelo motor
(`rationale_names`) e pelo caminho MCP de launch/grant. Próximo passo: achar o despacho
correto de `perms.rs` e testar a intrínseca diretamente.

## 4. Apêndice F — estado

| Item | Estado |
|---|---|
| Componentes activity, activity-alias, service, receiver, provider | Modelados (`ComponentKind`); testes em `appendix_f_tests` |
| Extensões: appwidget-provider, accessibility-service, notification-listener, voice-interaction, print-service, autofill-service, host-apdu, call-redirection, carrier-config, permission-group, permission-tree | Regras de extensão e `build_component` |
| Extensões: device-admin, wallpaper, dream, ime, media-browser, credentials | Já existiam antes deste trabalho (não re-auditadas nesta rodada) |
| **offline-device** | **Mapeado por decisão do usuário (2026-11-05), com ressalva.** Equivalente adotado: `android.app.extra.PROVISIONING_ALLOW_OFFLINE` (API 33, `DevicePolicyManager`), extra de `ACTION_PROVISION_MANAGED_DEVICE`. Ressalva: é extra de intent de provisionamento, não action de componente nem device feature, e nenhum manifest de app o declara. Por isso **não entra** na tabela `extension_kinds` (`crates/rd-apk/src/manifest.rs`), que só casa action de componente; a VM não implementa fluxo de provisionamento, então o status segue `not_implemented`. A página de offline-first (`developer.android.com/topic/architecture/data-layer/offline-first`) é de arquitetura e não define action nem elemento de manifest; a busca no `cs.android.com` não foi concluída nesta rodada. |
| Elementos de manifest (uses-permission, permission, uses-feature, queries, instrumentation, etc.) | Parse existente; não re-auditado item a item nesta rodada. |

Auditoria de extensões (rodada F-2, 2026-11-05), contra a documentação oficial do Android:

| Extensão | Achado | Correção |
|---|---|---|
| host-apdu | A action era o nome da classe; a oficial é `android.nfc.cardemulation.action.HOST_APDU_SERVICE` (`SERVICE_INTERFACE`) | Action trocada; teste com a action antiga como negativo |
| credentials | Exige `BIND_CREDENTIAL_PROVIDER_SERVICE` (API 34, signature); a regra não exigia | Regra exige a permissão; teste com e sem ela |
| appwidget-provider | `BIND_APPWIDGET` é concedida ao host (launcher); o provider não declara essa permissão. A regra exigia, então providers comuns não contavam | Regra sem permissão exigida; teste sem permissão |
| call-redirection, carrier-config, host-apdu (`BIND_NFC_SERVICE`) | Action e permissão confirmadas | Sem mudança |

Mutações das três correções: cada uma derruba o teste correspondente; arquivo restaurado e comparado byte a byte.
A permissão `BIND_CREDENTIAL_PROVIDER_SERVICE` não entrou em `data/permissions.toml`: a regra de extensão não lê essa tabela, e
adicioná-la muda a contagem de 175 que `table_matches_documented_counts` fixa junto com README e ROADMAP.
Essa mudança fica para uma rodada própria da tabela de permissões.

Auditoria de extensões (rodada F-3, 2026-11-05), para as regras que já existiam:

| Extensão | Achado | Correção |
|---|---|---|
| device-admin | A documentação oficial exige, além da ação `android.app.action.DEVICE_ADMIN_ENABLED` e de `BIND_DEVICE_ADMIN`, o `<meta-data android:name="android.app.device_admin">`. A plataforma falha com "No android.app.device_admin meta-data" sem ele (`DeviceAdminInfo`). A regra não exigia o meta-data | `build_component` só mantém `device-admin` com o meta-data; teste com e sem ele |
| wallpaper, ime | Ação e `BIND_WALLPAPER` / `BIND_INPUT_METHOD` confirmadas | Sem mudança |
| dream, media-browser, voice-interaction, print-service, autofill-service, call-screening, carrier-config | Ação e permissão conferidas com a tabela de regras; não reconfirmadas uma a uma na documentação nesta rodada | Sem mudança; pendente de checagem contra a documentação |

## 5. Apêndices C, D e E — plano por rodadas

Estes apêndices não são "completos" nesta rodada. Os números abaixo dizem o tamanho real.

### Estado das dependências de D (atualizado em 2026-11-05)

Medido no código (não na spec). Nenhum status de biblioteca mudou nesta rodada.

- **ST-04 (base de `security-crypto`, host-shim) — implementada em `crates/rd-vm/src/jkeystore.rs`:**
  provedor `AndroidKeyStore` (`KeyStore.getInstance`, `load`, `containsAlias`, `getKey`,
  `deleteEntry`); `KeyGenerator` AES com `KeyGenParameterSpec` (alias e tamanho 128/256);
  `Cipher` `AES/GCM/NoPadding` (`init` nos modos cifrar/decifrar, `getIV`, `updateAAD`,
  `update`, `doFinal`) com `aes-gcm` (cripto real, IV de 96 bits, tag de 128 bits);
  `GCMParameterSpec`; chave `AndroidKeyStoreSecretKey` sem material exportável
  (`getEncoded` = null). Erros tipados: `NoSuchAlgorithmException`, `KeyStoreException`,
  `InvalidKeyException`, `AEADBadTagException`. Testes: vetor NIST SP 800-38D (AES-128-GCM),
  round-trip pelos intrínsecos da VM, chave não exportável, falha após remoção.
- **Ainda não implementado em ST-04:** `MasterKey`, `EncryptedSharedPreferences` e
  `EncryptedFile` (dependem de Tink no security-crypto 1.0.0; a dependência da 1.1.0 não foi
  verificada); `KeyGenerator` AES fora do AndroidKeyStore (falha tipada); outros modos e
  paddings. `security-crypto` segue `not_implemented`: nenhum APK de teste validou o caminho DEX.
- **MEDIA-03 (dependência de `camerax`, host-shim):** a Camera2 virtual existe e está
  listada como suportada em `docs/m10.md` (`CameraManager`, `CameraDevice`,
  `CameraCaptureSession`, `ImageReader`). Falta a camada CameraX (`ProcessCameraProvider`,
  lifecycle, casos de uso `Preview`/`ImageCapture`/`ImageAnalysis`) e o modelo de cenas da spec.
  O APK do Comprix contém CameraX (`ProcessCameraProvider`, `PreviewView`) e serve de alvo de teste.
  - **Escolha do APK de teste (pesquisa no GitHub, 2026-11-05):** `arindamxd/camerax-android`
    (Apache-2.0, CameraX 1.6.1, Preview/ImageCapture/ImageAnalysis; o repositório foi renomeado
    e o antigo nome redireciona) não publica APK nas releases (1.7.0, 1.6.8, 1.6.7 sem assets) e
    exige build com Android SDK completo e JDK 17+ (o sandbox tem JDK 11 e só `platform-tools`).
    `carlosgub/CameraX-MLKIT_BarcodeScanning` não tem releases. Por isso o alvo continua sendo o
    APK do Comprix v1.6.0, já disponível localmente.
  - **Caminho de navegação até o scanner (medido):** `Pular` → `Nova lista` → chip
    `Compra do Mês` → `Criar e adicionar itens` → `Escanear um produto` → tela "Escanear preço"
    (modos Foto / Vídeo 30 s / Código). Sem permissão, a tela mostra "Câmera desligada" com
    `Permitir câmera`; tocar nesse botão não abre diálogo na VM. A permissão é concedida com
    `permissions_grant` (`android.permission.CAMERA`). Esperas de ~1–2,5 s após cada toque são
    necessárias: sem elas, `tap_text` falha com `VIEW_NOT_FOUND`.
  - **Bloqueio observado:** `ui_tree` não expõe texto do Compose; a navegação depende de
    `screenshot` e `tap_text` por rótulo visível.
  - **Diagnóstico (2026-11-05, com permissão concedida antes do launch):**
    1. `Context.getMainExecutor()` não tinha implementação; corrigido em #566 (`HandlerExecutor`).
       A `ClassCastException` Handler→Executor sumiu dos logs.
    2. Após o toque em "Escanear um produto", `LG2/C.d` (scanner) executa até criar a
       `PreviewView` e chamar `setScaleType`/`redrawPreview`. Não há `addView` depois disso, e
       `ui_tree`/`observe` continuam mostrando a lista ("unchanged since last observe").
    3. Só 15 chamadas de `Lbr/` depois do toque: não há laço de Kotlin. O host registra 97
       `Activity.getPackageName()` e 108 `ViewGroup.getChildAt()`; `fuel` cresce durante a espera.
    4. Medido com `RD_TRACE_DRAW` e `RD_TRACE_METHOD` (2026-11-05, sessão seguinte):
       - O laço de acessibilidade (`LK0/`, `getPackageName`, `getChildAt`) já existe na tela
         de lista, antes do toque. Não é a causa.
       - O frame do Compose é agendado pelo `AndroidUiDispatcher` (`LB0/k0`, que é
         `FrameCallback` e `Runnable`). Após o toque, a composição do scanner (`LG2/`) roda
         dentro de dois frames em 8752 ms, seguida de ~18.000 chamadas `LP/` (Composer/Recomposer).
       - Ao fim desse trabalho, `run()` do dispatcher remove o callback com `removeFrameCallback`
         (fila de tarefas vazia). Nenhum frame novo é pedido depois, mesmo com `wait`.
       - Não há `addView`/`requestLayout` que insira a `PreviewView` na árvore (o
         `AndroidViewsHandler` nunca recebe o holder). Assim a árvore continua na lista.
       - Ordem medida com `RD_TRACE_METHOD=L` (trace completo da janela após o toque):
         1. Frame 1 (8752 ms): `LP/u0.p` (recompõe a árvore atual), `c()`, `r()` e
            `LP/t.e(LQ/a;)V` (`applyChanges`, rótulo `Compose:applyChanges`). Esse ciclo completa.
         2. Frame 2 (8752 ms): `LP/u0.p` inicia a composição que contém o scanner;
            `LG2/C.d` executa (~99 mil chamadas depois). Depois disso não há `c()`, `r()` nem um
            segundo `applyChanges`. O callback é removido em `removeFrameCallback` (133 mil).
         - Exceções: nenhuma `Throwable` lançada na janela (só `exceptionOrNull`).
         - Depois do frame 2, a corrotina do scanner recebe cancelamentos (`Le4/k0.f(CancellationException)`,
           6 ocorrências após o `removeFrame`, a primeira em ~139 mil chamadas). Não há cancelamento
           entre os frames 1 e 2. Próximo passo: descobrir qual job é cancelado (lançado em `Ls0/c.V`/`K1/l`)
           e por quem; é provável que o Recomposer seja cancelado antes do apply.
       - Hipótese principal (revisada): a composição do scanner termina sem o caminho normal de
         retorno de `LP/u0.p`, então o Recomposer não aplica as mudanças. Por isso a
         `PreviewView` não é inserida. Próximo passo: verificar por que `LP/u0.p` do frame 2
         não retorna para `c()`/`r()` (ex.: exceção suprimida, `abandon` ou cancelamento da
         corrotina), comparando com o frame 1. Só então partir para o shim CameraX.
- **Decisões do usuário (2026-11-05):** ST-04 com crate AES-GCM real (aprovado); MEDIA-03:
  o usuário autorizou escolher um APK de teste no GitHub; latência: parar e registrar a lacuna.

### Apêndice C (literal)
- **Bloqueio fundamental:** sem `android.jar` não há inventário literal. Rodada 0:
  obter um `android.jar` de API 26–36 (repositório Maven do Google, `com.google.android:android`),
  gerar a lista de classes/métodos públicos por prefixo registrado e gravar o total em
  `docs/appendix-c-inventory.md`. Nenhuma superfície muda de status antes disso.
- Rodadas seguintes: por domínio, implementar ou marcar bloqueio por item, na ordem do
  maior número de superfícies `partial` (ver tabela da spec, domínios App/lifecycle e View/UI).

### Apêndice D (AndroidX)
- 20 bibliotecas `not_implemented`. Prioridade pelo critério da spec ("Room via ST-02,
  work via SYS-04, navigation via UI-04, ..."): sem estratégia definida para as 20. Rodada 1:
  escolher estratégia por biblioteca (`dex-passthrough` quando o DEX da lib já roda na VM;
  `host-shim` quando precisa de interceptação) e registrar como `planned` no `notes`,
  sem flipar para `supported`.
- 7 `stub`: promover só com teste de comportamento real.

### Apêndice E (GMS/Firebase)
- 20 serviços `partial`; ML Kit `not_implemented` (é extra, fora da tabela da spec).
  Rodada 1: decidir se ML Kit entra na spec (usuário). Demais serviços: critério de
  shim programável com teste de sucesso, erro e ciclo de vida.

## 6. Decisões que dependem do usuário

1. **offline-device (F): resolvido pelo usuário em 2026-11-05** — mapeado para `PROVISIONING_ALLOW_OFFLINE` com a ressalva da §4. Se houver uma action de device feature oficial, substituir a linha da §4.
2. **revokeWhenRequested, instant, softRestriction (B):** fornecer a fonte de semântica, ou aceitar como bloqueio documentado.
3. **Apêndice C literal:** aceitar que o inventário depende de `android.jar` (obtenção pela rede do sandbox) ou fornecer o arquivo.
4. **Escopo de D e E:** quais das 20 bibliotecas D `not_implemented` e do serviço ML Kit entram nesta spec.

## 7. Validação deste plano

- `cargo test -p rd-framework`, `-p rd-agent`, `-p rd-cli`, `-p rd-vm`, `-p rd-apk`: verdes no PR.
- `cargo fmt --all -- --check` e `cargo clippy --all-targets` nos crates tocados.
- `rd mcp --gen-docs docs/mcp-tools.md` regenerado; teste `generated_docs_match_catalog` verde.
- Matriz nativa (`scripts/native-languages-mcp.py`) exige 52 tools com chamada positiva; roda no CI com NDK.
