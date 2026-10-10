# compat-matrix — placar oficial do RustyDroid

> **GERADO** por `cargo xtask compat-report` — não editar à mão.
> Lei de Ouro: nenhuma API ausente quebra o app em silêncio — tudo responde `NOT_IMPLEMENTED(id)` e aparece aqui.

## Estado por fase (roadmap M0–M12)

| Fase | Entrega | Status |
|---|---|---|
| M0 | workspace · rd-apk inspect · motor de permissões · devices · behavior | **✅ implementado** |
| M1 | rd-dex 100% + disassembler | **✅ implementado** |
| M2 | VM Dalvik mínima | **✅ implementado** |
| M3 | framework essencial headless | **✅ implementado (rounds 1+2: lifecycle/touch + LayoutInflater XML)** |
| M4 | render + UI dump | ⬜ |
| M5 | agente MCP v1 (~20 tools) | ⬜ |
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
| `rd-render` | M4 | stub NOT_IMPLEMENTED |
| `rd-agent` | M5 | stub NOT_IMPLEMENTED |
| `rd-jni` | M3+ | stub NOT_IMPLEMENTED |
| `rd-ndk` | M9 | stub NOT_IMPLEMENTED |

## Dados versionados (M0)

- permissions.toml: **175** permissões + 13 roles (Apêndice B)
- devices.toml: **6** perfis (piso: `moto-e5`)
- behavior-switches.toml: **25** comutadores targetSdk 26→36 (PRF-07)

## Compatibilidade por módulo (checklist PARTE 0)

- Núcleo (0.0): parcial (CORE-01..06 M0; rd-dex M1; VM M2 — métodos puros executam)
- Framework (M3): Activity lifecycle + views host + Handler/Looper + touch + **LayoutInflater de layout XML real** (setContentView(I): resid → @layout/key no arsc → AXML do APK → árvore host; attrs id/text/orientation/visibility/enabled/onClick/layout_height; setText(I), getString(I), findViewById) — classes de view sem host (ImageView/…) e configs específicas (land/locale) respondem tipadas
- Render/agente: `NOT_IMPLEMENTED(module_id)` estruturado (M4+)
