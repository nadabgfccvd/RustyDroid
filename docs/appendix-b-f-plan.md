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
| `jetpack.toml` (D) | 46 bibliotecas, 6 categorias | 25 partial · 3 stub · 18 not_implemented (medido em 2026-11-06) |
| `gms.toml` (E) | 21 serviços, 3 categorias | 20 partial · 1 not_implemented (ML Kit, fora de escopo por decisão do usuário) |
| `permissions.toml` (B) | 195 permissões, 13 roles | 5 com `hardRestricted`; 5 com `role = "SMS"` (após este PR) |
| Catálogo MCP | 52 tools | inclui `permissions_auto_revoke` e `permissions_role` (Apêndice B6) |

O inventário literal de C está medido e commitado em `docs/appendix-c-inventory.md`
(API 26–36, gerado de `android.jar` oficiais). Em 2026-11-06 os 11 jars foram baixados
de `dl.google.com/android/repository` (pacotes `platform-NN`; o Maven do Google só publica
`com.google.android:android` 4.1.1.4) e o script `scripts/appendix_c_inventory.py`
regenerou os CSVs e o Markdown byte a byte iguais aos commitados. Ver §5, Apêndice C.

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
| Flag `softRestriction` (2a) — política da plataforma para READ/WRITE_EXTERNAL_STORAGE | `soft_may_grant`, `soft_weak_form`, recusa em `grant()` (`soft_blocked`); flag em `data/permissions.toml` para as duas permissões | `soft_restricted_storage_is_refused_below_q_without_allowlist`; `soft_restricted_storage_is_granted_in_full_from_api_29`; `soft_restricted_permission_without_platform_rule_keeps_full_grant`. Mutação (sem a recusa) derruba o primeiro teste. |
| Flag `instant` + modo de app instantâneo (2c) | flag `instant` em 26 permissões do `core/res` do AOSP (`data/permissions.toml`); `install` com `instant` (opt-in, default false) → `PermissionEngine::set_instant`: permissão sem a flag vira `DeniedSystem`; `grant`, `set_role`, `grant_partial_media` e `update` respeitam; `isInstantApp` lê `Engine::instant_app` | `instant_flag_in_table_matches_framework_manifest`; `instant_app_keeps_only_instant_capable_grants`; `instant_app_cannot_be_granted_non_instant_permission`; `instant_rule_applies_to_permissions_requested_by_update`; `instant_app_cannot_receive_non_instant_role_permissions`; `install_as_instant_app_limits_grants_and_fixes_mode` (MCP); `is_instant_app_follows_install_mode` (VM) |
| `revokeWhenRequested` + split permissions (2b) | `[[split]]` em `data/permissions.toml` (tabela do `platform.xml`); `implicit_permissions`, `apply_implicit` no `install`; `PermissionEngine::update` (ciclo de update, chamado pelo `install` do agente quando o pacote já existe); `grant` limpa a flag | `implicit_grant_is_revoked_when_updated_app_requests_it_explicitly`; `implicit_grant_is_kept_while_app_still_does_not_request_it`; `explicit_request_below_m_drops_flag_without_revoking`; `nearby_devices_grant_is_preserved_when_background_location_is_granted`; `user_grant_clears_revoke_when_requested_flag`; `split_implicit_permissions_depend_on_target_sdk` |

### Resolvido após a pesquisa (2026-11-07, PRs #581–#583)

Os três bloqueios desta seção foram resolvidos com as decisões do usuário (§6, itens 2a–2c).
A pesquisa original (fontes e pré-requisitos) continua abaixo, como registro.

| Flag | Decisão | Implementação | Testes |
|---|---|---|---|
| `softRestriction` (2a) | Forma fraca decidida pela pesquisa: política da plataforma (`SoftRestrictedPermissionPolicy`) só para `READ/WRITE_EXTERNAL_STORAGE`; demais permissões mantêm concessão plena | `soft_may_grant`, `soft_weak_form`, recusa em `grant()` (`soft_blocked`); flag em `data/permissions.toml` (#581) | `soft_restricted_storage_is_refused_below_q_without_allowlist`; `soft_restricted_storage_is_granted_in_full_from_api_29`; `soft_restricted_permission_without_platform_rule_keeps_full_grant` |
| `revokeWhenRequested` (2b) | Aprovado: ciclo de update de pacote e conjunto de permissões implícitas por versão | `[[split]]` em `data/permissions.toml` (tabela do `platform.xml`); `implicit_permissions`, `apply_implicit` no `install`; `PermissionEngine::update`; `grant` limpa a flag (#582) | `implicit_grant_is_revoked_when_updated_app_requests_it_explicitly`; `implicit_grant_is_kept_while_app_still_does_not_request_it`; `revoke_when_requested` |
| `instant` (2c) | Aprovado: modelo de app instantâneo, opt-in por instalação (default falso) | Flag `instant` em 26 permissões do `core/res` do AOSP; `PermissionEngine::set_instant`; permissão sem a flag vira `DeniedSystem`; `grant`, `set_role`, `grant_partial_media` e `update` respeitam; `isInstantApp` lê `Engine::instant_app` (#583) | `instant_app_cannot_be_granted_non_instant_permission`; `instant_app_keeps_only_instant_capable_grants`; `instant_app_cannot_receive_non_instant_role_permissions`; `instant_flag_in_table_matches_framework_manifest`; `instant_rule_applies_to_permissions_requested_by_update` |

Desvio conhecido: `isInstantApp` só muda quando o VM é configurado como app instantâneo;
o padrão continua `false`, então apps existentes não mudam de comportamento.

#### Segunda rodada de pesquisa (2026-11-06): fontes lidas e pré-requisitos

A pesquisa anterior parou no trecho de revogação de `PermissionManagerServiceImpl.java`. Nesta
rodada o arquivo inteiro (5368 linhas, `services/core/java/com/android/server/pm/permission/`,
branch `main`) e `PackageManager.java` foram baixados de `android.googlesource.com` (`?format=TEXT`)
e lidos. Cópias locais fora do repositório: `/home/user/appc/aosp/`.

- **`revokeWhenRequested` (fonte completa do comportamento):**
  - Doc da flag (`PackageManager.java:5335`): "The permission has not been explicitly requested by the
    app but has been added automatically by the system. Revoke once the app does explicitly request it."
  - Revogação em `revokePermissionsNoLongerImplicitLocked` (PMSI ~2984–3045). Chamada em
    `updatePermissions` de cada pacote (~2953), com o conjunto de permissões implícitas do pacote.
    Revoga só se: permissão concedida e runtime; flag `REVOKE_WHEN_REQUESTED` setada; não está em
    `BLOCKING_PERMISSION_FLAGS` (system fixed, policy fixed, granted by default); target SDK do UID >= 23;
    e não é a preservação de "Nearby devices" com `ACCESS_BACKGROUND_LOCATION` concedida. Ao final,
    a flag é removida sempre; as flags do usuário só se houve revogação.
  - A flag é setada na migração de permissão split (PMSI ~3188), exceto `ACTIVITY_RECOGNITION` e
    permissões de mídia aural/visual. Herança em ~4800–4870 (merge de flags entre fontes).
  - **Pré-requisito que falta no VM:** não existe ciclo de update de pacote (o motor é construído
    uma vez em `PermissionEngine::install`) e não há conjunto de permissões implícitas por versão.
    Sem o gatilho de update, a função de revogação não teria chamador. Adicionar esse ciclo muda o
    comportamento de todo app com permissão split, então precisa de decisão do usuário (ver §6).
- **`instant` (fonte completa do comportamento no framework):**
  - `checkSinglePermissionInternalLocked` (PMSI ~988–999): para app instantâneo, uma permissão concedida
    só conta se `Permission.isInstant()`.
  - `grantRuntimePermission` (PMSI ~1470–1472): concessão de permissão não-instant a app instantâneo
    lança `SecurityException` ("Cannot grant non-ephemeral permission").
  - **Pré-requisito que falta no VM:** não existe estado de app instantâneo. `isInstantApp` é fixo em
    `false` (`android_pm.rs:721`). Modelar o estado muda a resposta de `isInstantApp` para
    qualquer app que o VM hospede, então também precisa de decisão do usuário.
- **`softRestriction` (concessão, fonte parcial):**
  - `grantRuntimePermission` (PMSI ~1378–1390 e ~1456): permissão soft restricted só é concedida se
    `SoftRestrictedPermissionPolicy.forPermission(...).mayGrantPermission()` for verdadeiro; senão a
    concessão é recusada ("Cannot grant soft restricted permission"). A própria política
    (`services/core/java/com/android/server/policy/SoftRestrictedPermissionPolicy.java`) ainda não foi lida.
  - O VM não tem nenhuma modelagem de soft restriction (`rd-framework/src/fd_permissions.rs`).
    A forma fraca continua na pergunta §6 item 2a.

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
  - **Reprodução em 2026-11-06 (build de `origin/main` 47e0e26, trace `RD_TRACE_METHOD=L`):**
    - A aba **Scanner** da barra inferior não é a rota diagnóstica: o toque (por rótulo ou por
      coordenada 360,1367) é aceito, mas a tela volta para "Minhas listas" (após Ajustes, o toque
      em Scanner também cai em Listas). Não investigar essa aba como se fosse o scanner de MEDIA-03.
    - Rota reproduzida: `Pular` → `Nova lista` → `Compra do Mês` → `Criar e adicionar itens` →
      `Escanear um produto`. O trace mostra `LG2/C.d` executado uma vez (como no diagnóstico
      acima); o `observe` fica em `frame v5` sem mudança e `find_views` por `PreviewView` retorna 0.
      O toque em "Escanear um produto" é aceito (`listener_acionado`), mas a tela não avança.
    - Harness recriado (não está mais em `/tmp`): `/home/user/appc/camx/lib.py` (classe `S`) e
      `route.py` e `route_obs.py` (scripts da rota acima) em `/home/user/appc/camx/`.
      Fora do repositório; reexecutar a partir deles se a pasta existir.
    - Próximo passo (não feito): identificar o job cancelado (`Ls0/c.V`/`K1/l`) e a origem do
      `Throwable` do frame 2, comparando o trace desta rota com `LP/u0.p`.
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
       - Hipótese principal (SUPERADA pela causa raiz 6 abaixo): a composição do scanner termina sem o caminho normal de
         retorno de `LP/u0.p`, então o Recomposer não aplica as mudanças. Por isso a
         `PreviewView` não é inserida. Próximo passo: verificar por que `LP/u0.p` do frame 2
         não retorna para `c()`/`r()` (ex.: exceção suprimida, `abandon` ou cancelamento da
         corrotina), comparando com o frame 1. Só então partir para o shim CameraX.
   6. **Causa raiz (2026-11-06, substitui a hipótese do Recomposer acima):** a composição do
      scanner não abortava por causa do Recomposer. Cada bloqueio era uma classe ou
      configuração ausente, que lançava exceção e cancelava a corrotina. Em ordem:
      - `android.view.ScaleGestureDetector`: `NOT_FOUND` no `<init>` (`f918dc0`). Modelo com
        ponteiro único; pinch e multi-toque ficam como limite documentado.
      - `Resources$NotFoundException` sem hierarquia: virava `ClassCastException` fatal
        (`6523e18`, `=> RUNTIME`).
      - `android.R.color.black` (`0x0106000c`): `NotFoundException` do `PreviewView`
        (`0efea59`, tabela `FRAMEWORK_COLORS`, valores de developer.android.com/reference/android/R.color).
      - `metaData` de componente sempre `null`: CameraX sem `CameraXConfig.Provider`
        (`e063841`, aprovado pelo usuário em 2026-11-06 e mesclado na PR #575). Confirmado por trace e por
        `rd dex disasm`: `ComprixApp` não implementa a interface.
      - `ThreadPoolExecutor` sem modelo (`fe73736`, executor inline; setters aceitos, getters não modelados).
      - `HandlerThread` sem modelo: `NOT_FOUND` no construtor (`8a0b296`). Uma Looper por thread, com a
        mesma fila da principal (`handler_thread.rs`). Corpo da thread = drenagem (desvio documentado).
      - `java.util.logging.Logger.getLogger`: o `AbstractFuture` do Guava empacotado no APK chama isso no
        inicializador estático; `NOT_IMPLEMENTED` virava `ExceptionInInitializerError` fatal (`63637b3`).
        Modelo do subconjunto de String, conforme `AndroidHandler` da AOSP.
   7. **Estado após a correção 6 (build `rd-cli` de 2026-11-06, `build_rd11.log`):**
      - `last_error` = `null` (antes: `JAVA_EXCEPTION`, depois `NOT_FOUND`). Nenhum erro fatal na VM.
      - Scanner renderiza: "Escanear preço", botões Foto/Vídeo/Código, "Tirar foto" (`scanner_sg_fix.png`).
      - `find_views` com `class=PreviewView`: 1 match (`androidx.camera.view.PreviewView`, 720x1016 em y=230).
        Critério de "PreviewView > 0" cumprido.
   8. **Ainda não cumprido:** o preview sai preto. O CameraX repete "Retry init" com
      `NullPointerException` (logs de 2026-11-06). Não há câmera ligada ao CameraX: falta o caminho
      de `Camera2` (`CameraManager`, `CameraDevice`, `ImageReader`) e um dispositivo de câmera no VM.
      Esse é um modelo de hardware, e a escolha de um feed virtual (ou de câmera vazia) é decisão do
      usuário. **Decisão (2026-11-06): opção (a), Camera2 com câmera virtual**, que muda o comportamento
      de todos os apps com câmera. Ainda não implementado. Próximo passo: identificar a origem do NPE do
      "Retry init" com o trace de `CameraX`/`Camera2`, e só então implementar.
   9. **Limite de ambiente (2026-11-06):** a sandbox tem ~2 GB de RAM e sem swap. O `rustc` do `rd-vm`
      estoura a memória (OOM) mesmo com `CARGO_BUILD_JOBS=1`, por causa do `src/android_stub_surfaces.rs`
      (2,1 MB, código escrito à mão). Por isso não foi possível compilar `rd-cli` nem rodar o Scanner
      local. Validação passa a ser pelo CI, e a compilação local, só com `cargo check` ou
      `xtask`. O restante do MEDIA-03 (câmera) depende desse ambiente ou de um binário de CI.

- **Decisões do usuário (2026-11-05):** ST-04 com crate AES-GCM real (aprovado); MEDIA-03:
  o usuário autorizou escolher um APK de teste no GitHub; latência: parar e registrar a lacuna.

### Apêndice C (literal)
- **Rodada 0 — concluída (2026-11-06):** o `android.jar` de API 26–36 não está no Maven
  do Google para essas APIs (`com.google.android:android` tem só 4.1.1.4). Os 11 jars
  oficiais vieram dos pacotes `platform-NN` em `dl.google.com/android/repository`
  (26: `platform-26_r02`, 27: `platform-27_r03`, 28: `platform-28_r06`, 29: `platform-29_r05`,
  30: `platform-30_r03`, 31: `platform-31_r01`, 32: `platform-32_r01`, 33: `platform-33-ext3_r03`,
  34: `platform-34-ext7_r03`, 35: `platform-35_r02`, 36: `platform-36_r02`).
  `scripts/appendix_c_inventory.py --sdk-dir <dir>` reproduz `docs/appendix-c-inventory.md`
  e `docs/api-inventory/*.csv` sem diferença. Nenhuma superfície mudou de status com isso.
  Contagem do script: 4181 arquivos de classe no API 26 e 6227 no API 36 (brutos); o
  Markdown reporta 4172 e 6222 classes públicas + protegidas.
- **Verificação de fonte (2026-11-06, pedido do usuário "baixar do Maven do Google"):**
  `maven.google.com` e `dl.google.com/dl/android/maven2` respondem 404 para
  `com/google/android/android/` (metadados e o POM de 4.1.1.4). O artefato só existe no
  Maven Central (`repo1.maven.org`), com versões até `4.1.1.4` (API 4.1, sem nenhuma API 26+).
  Por isso APIs 26–36 vieram dos pacotes `platform-NN` oficiais (`dl.google.com/android/repository`),
  que é a fonte oficial do `android.jar`. Substituição de fonte registrada aqui para decisão do usuário.
- **Próximo passo de C:** a lacuna é de implementação, não de inventário. Por domínio, com
  maior número de superfícies `partial` primeiro (`android-view`, `android-widget`,
  `android-os`, `android-content`, `android-graphics`), implementar ou marcar bloqueio por item.
- Rodadas seguintes: por domínio, implementar ou marcar bloqueio por item, na ordem do
  maior número de superfícies `partial` (ver tabela da spec, domínios App/lifecycle e View/UI).

### Apêndice D (AndroidX)
- Estado medido em 2026-11-06 (`data/jetpack.toml`, 46 bibliotecas): 25 `partial`, 3 `stub`,
  18 `not_implemented`. Os números anteriores (19/7/20) eram de antes das rodadas S9–S12 de
  `docs/appendix-d-e.md`.
- Escopo decidido pelo usuário: Room (via ST-02) e as bibliotecas de maior prioridade pelo
  critério da spec. ML Kit fica fora.
- Prioridade da spec ("Room via ST-02, work via SYS-04, navigation via UI-04"): as três já estão
  `partial` com `host-shim`. Lacunas que restam:
  - Room: DAOs gerados por annotation processing, migrations e `InvalidationTracker` não são
    modelados (`docs/jetpack.md`, linha room).
  - work e navigation: ver o campo `notes` de cada entrada em `data/jetpack.toml`.
- Entre as 18 `not_implemented`, o ST-04 (AES-GCM no AndroidKeyStore, PR #564) não muda o
  status de `security-crypto`: `EncryptedSharedPreferences` e `MasterKey` dependem de Tink, e
  nenhum APK de teste validou o caminho DEX. Continua `not_implemented`.
- Próximo passo: escolher, entre as 18, a primeira biblioteca com estratégia pronta e teste de
  comportamento real. Não promover nenhuma sem teste.
- 3 `stub` (ConstraintLayout, CoordinatorLayout, material): promover só com teste de comportamento real.

### Apêndice E (GMS/Firebase)
- 20 serviços `partial`. ML Kit `not_implemented` (extra, fora da tabela da spec) fica fora de
  escopo por decisão do usuário (2026-11-05).
- Demais serviços: critério de shim programável com teste de sucesso, erro e ciclo de vida.

## 6. Decisões que dependem do usuário

1. **offline-device (F): resolvido pelo usuário em 2026-11-05** — mapeado para `PROVISIONING_ALLOW_OFFLINE` com a ressalva da §4. Se houver uma action de device feature oficial, substituir a linha da §4.
2. **revokeWhenRequested, instant, softRestriction (B):** resolvido pelo usuário (autonomia para pesquisar e implementar com base em fonte oficial). Resultado da pesquisa na §3.
2a. **softRestriction (B), forma fraca: resolvido por pesquisa (autonomia do usuário, 2026-11-07).** Fonte da regra: AOSP `SoftRestrictedPermissionPolicy.java` (regras só para READ/WRITE_EXTERNAL_STORAGE; demais = concessão plena). Forma fraca: READ sem allowlist e target < 29 = só coleções de mídia visual/áudio (doc oficial de `READ_EXTERNAL_STORAGE`); WRITE sem allowlist e target < 29 = sem acesso (grant recusado). Efeito: apps com target < 29 que pedem storage deixam de receber a forma completa pelo runtime sem allowlist. Mudança registrada no PR de 2a.
2e. **ACCESS_BACKGROUND_LOCATION: flag divergente da doc oficial (pergunta ao usuário).** O registro marca `softRestriction`; a doc oficial (`Manifest.permission`, API 29) diz que é **hard restricted** (só concedível com allowlist do instalador). Corrigir a flag muda o grant de todo app que pede localização em background, então não foi feito sem decisão. Hoje a flag não muda o comportamento (sem regra de política = concessão plena).
2b. **revokeWhenRequested (B): ciclo de update de pacote — aprovado pelo usuário (2026-11-07) e implementado.**
   Gatilho: `install` do agente com pacote já instalado chama `PermissionEngine::update` (antes recriava o motor
   e zerava as permissões). Escopo: splits de `platform.xml` (exceto BODY_SENSORS → `health.*`, que depende de
   feature flag). Para targetSdk < 23 a implícita concedida é aplicada na instalação (modelagem nossa, o AOSP
   concede na instalação; explícitas pré-M seguem `RuntimePending`). Efeito colateral aceito: apps com target < 33
   que pedem WRITE/READ ganham entradas `READ_MEDIA_*` no estado.
2c. **instant (B): estado de app instantâneo — aprovado pelo usuário (2026-11-07) e implementado.**
   Modelo: opt-in na instalação (`install` com `instant: true`), que é o equivalente de `adb install --instant`.
   Apps instalados sem a opção não mudam (`isInstantApp` segue `false` para eles). Fonte da lista de
   permissões com `instant`: `core/res/AndroidManifest.xml` do AOSP `main` (28 entradas; 26 existem na tabela,
   faltam `INSTANT_APP_FOREGROUND_SERVICE` e `FOREGROUND_SERVICE_FILE_MANAGEMENT`, lacuna de dado registrada).
   O modo não muda em update (erro `INSTANT_MODE_CHANGE`). Não modelado: o cache de instalação instantânea
   e o `PackageInstaller` de app instantâneo.
2d. **MEDIA-03 (câmera): decisão de escopo tomada em 2026-11-06** — Camera2 com câmera virtual (opção a),
   que altera todos os apps com câmera. **Implementado e mesclado (#577, #578, #579).** Fechamento validado
   com o Scanner: `last_error` vazio, sem `Retry init`, `PreviewView` com frames (screenshot não preta).
   Semânticas aprovadas: `ThreadLocal` por green thread e executores que honram `ThreadFactory`.
   Limite documentado: o detector de código de barras do ML Kit é `host_stub` (Task concluído com lista
   vazia; sem leitura de código de barras no VM), por decisão do usuário.
3. **Apêndice C literal:** resolvido em 2026-11-06 — o `android.jar` foi obtido pela rede do sandbox (ver §5, Apêndice C).
4. **Escopo de D e E: resolvido pelo usuário (2026-11-05)** — Room (via ST-02) e prioridade da spec; ML Kit fora (ver §5).

## 7. Validação deste plano

- `cargo test -p rd-framework`, `-p rd-agent`, `-p rd-cli`, `-p rd-vm`, `-p rd-apk`: verdes no PR.
- `cargo fmt --all -- --check` e `cargo clippy --all-targets` nos crates tocados.
- `rd mcp --gen-docs docs/mcp-tools.md` regenerado; teste `generated_docs_match_catalog` verde.
- Matriz nativa (`scripts/native-languages-mcp.py`) exige 52 tools com chamada positiva; roda no CI com NDK.
