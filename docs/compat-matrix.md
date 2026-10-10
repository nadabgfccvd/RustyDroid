# compat-matrix — placar oficial do RustyDroid

> **GERADO** por `cargo xtask compat-report` — não editar à mão.
> Lei de Ouro: nenhuma API ausente quebra o app em silêncio — tudo responde `NOT_IMPLEMENTED(id)` e aparece aqui.

## Estado por fase (roadmap M0–M12)

| Fase | Entrega | Status |
|---|---|---|
| M0 | workspace · rd-apk inspect · motor de permissões · devices · behavior | **✅ implementado** |
| M1 | rd-dex 100% + disassembler | **✅ implementado** |
| M2 | VM Dalvik mínima | **✅ implementado** |
| M3 | framework essencial headless | ⬜ |
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
| `rd-cli` | M0–M2 | inspect · perm · device · behavior · dex disasm · vm exec (contrato de erro JSON) |
| `rd-dex` | M1 | parser DEX completo + disassembler smali validado vs baksmali + fuzz targets |
| `rd-vm` | M2 | interpretador Dalvik mínimo — métodos puros de APK real (golden harness vs JVM) + intrinsics |
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
- Framework/render/agente: `NOT_IMPLEMENTED(module_id)` estruturado (M3+)
