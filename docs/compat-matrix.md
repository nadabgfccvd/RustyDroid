# compat-matrix — placar oficial do RustyDroid

> **GERADO** por `cargo xtask compat-report` — não editar à mão.
> Lei de Ouro: nenhuma API ausente quebra o app em silêncio — tudo responde `NOT_IMPLEMENTED(id)` e aparece aqui.

## Estado por fase (roadmap M0–M12)

Legenda (regra de status, ROADMAP): ✅ concluído = DoD reproduzível a partir do repositório e executado no CI ou em campanha documentada · ⚠️ parcial = código presente, mas DoD incompleto · ⬜ não iniciado.

| Fase | Entrega | Status |
|---|---|---|
| M0 | workspace · rd-apk inspect · motor de permissões · devices · behavior | **⚠️ parcial** — DoD exige 5 APKs reais (há 1, F-Droid, minSdk 24 < piso 26) |
| M1 | rd-dex 100% + disassembler | **⚠️ parcial** — parser+disasm no código; fuzz 24 h nunca executado; comparador baksmali não versionado |
| M2 | VM Dalvik mínima | **✅ concluído** — golden 150 vetores vs JVM no CI (job vm-golden) |
| M3 | framework essencial headless | **⚠️ parcial** — lifecycle/touch/LayoutInflater no código; prova com 3 APKs reais pendente (só sintéticos) |
| M4 | render + UI dump | **⚠️ parcial** — dump/screenshot ativos; sem tiny-skia (ver ADR 003), sem seleção por device-config, sem golden Android |
| M5 | agente MCP v1 (30 tools + controles) | **✅ concluído** — e2e com binário em processo separado + catálogo == describe-tools == docs (testes no CI) |
| M6 | executor de testes + compat automatizada | ⬜ |
| M7 | performance + gate piso E5 | ⬜ |
| M8 | compat real (views/rede/dados) | ⬜ |
| M9 | componentes avançados + NDK | ⬜ |
| M10 | dispositivos virtuais | ⬜ |
| M11 | engines (Flutter/RN/Mono/wasm) | ⬜ |
| M12 | ecossistema GH completo | ⬜ |

## Crates

| Crate | Fase | Escopo |
|---|---|---|
| `rd-apk` | M0 | ZIP + AXML + arsc + assinaturas + manifest model completo |
| `rd-framework` | M0 | fd-permissions (motor completo + gating maxSdk/since_api) · fd-devices · fd-behavior |
| `rd-cli` | M0–M3 | inspect · perm · device · behavior · dex disasm · vm exec · app run (contrato de erro JSON) |
| `rd-dex` | M1 | parser DEX completo + disassembler smali validado vs baksmali + fuzz targets |
| `rd-vm` | M2–M3 | interpretador Dalvik + golden harness vs JVM + framework host headless (Activity/View/Handler/Looper) + LayoutInflater AXML/ARSC |
| `rd-render` | M4 | UI dump uiautomator + screenshot por software (glifos de bloco; tiny-skia/shaping pendentes — ver ADR 003); winit/web = NOT_IMPLEMENTED (M11) |
| `rd-agent` | M5 | servidor MCP stdio (JSON-RPC, ADR 001) com 30 tools + controles de sandbox/limites |
| `rd-jni` | M3+ | stub NOT_IMPLEMENTED |
| `rd-ndk` | M9 | stub NOT_IMPLEMENTED |

## Dados versionados (M0)

- permissions.toml: **175** permissões + 13 roles (Apêndice B)
- devices.toml: **6** perfis (piso: `moto-e5`)
- behavior-switches.toml: **25** comutadores targetSdk 26→36 (PRF-07)

## Compatibilidade por módulo (checklist PARTE 0)

- Núcleo (0.0): parcial (CORE-01..06 M0; rd-dex M1; VM M2 — métodos puros executam)
- Parser DEX: entende o formato (M1). Execução de bytecode: VM Dalvik (M2) — coisas distintas; passar no parser não implica execução.
- Framework (M3): Activity lifecycle + views host + Handler/Looper + touch + LayoutInflater de layout XML (setContentView(I): resid → @layout/key no arsc → AXML do APK → árvore host; attrs id/text/orientation/visibility/enabled/onClick/layout_height; setText(I), getString(I), findViewById). Views sem host (ImageView/…) respondem INFLATE tipado.
- Seleção de recursos por configuração (M4): NÃO implementada. `resolve_string`/`resolve_value` escolhem a config mais genérica (default), sem locale/orientação/densidade/night do device; o inflater usa `res/layout/<key>.xml` e, na ausência, um fallback pontuado (`-land`/`-night`).
- Render (M4): dump uiautomator + screenshot por software; texto com glifos de bloco 3×7 (sem fonte/shaping). Agente (M5): `NOT_IMPLEMENTED(module_id)` só em winit/web (M11), NDK (M9) e JNI.
