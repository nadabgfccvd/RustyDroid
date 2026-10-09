# compat-matrix — placar oficial do RustyDroid

> **GERADO** por `cargo xtask compat-report` — não editar à mão.
> Lei de Ouro: nenhuma API ausente quebra o app em silêncio — tudo responde `NOT_IMPLEMENTED(id)` e aparece aqui.

## Estado por fase (roadmap M0–M12)

| Fase | Entrega | Status |
|---|---|---|
| M0 | workspace · rd-apk inspect · motor de permissões · devices · behavior | **✅ implementado** |
| M1 | rd-dex 100% + disassembler | ⬜ |
| M2 | VM Dalvik mínima | ⬜ |
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
| `rd-framework` | M0 | fd-permissions (motor completo) · fd-devices · fd-behavior |
| `rd-cli` | M0 | inspect · perm · device · behavior (contrato de erro JSON) |
| `rd-dex` | M1 | stub NOT_IMPLEMENTED |
| `rd-vm` | M2 | stub NOT_IMPLEMENTED |
| `rd-render` | M4 | stub NOT_IMPLEMENTED |
| `rd-agent` | M5 | stub NOT_IMPLEMENTED |
| `rd-jni` | M3+ | stub NOT_IMPLEMENTED |
| `rd-ndk` | M9 | stub NOT_IMPLEMENTED |

## Dados versionados (M0)

- permissions.toml: **173** permissões + 13 roles (Apêndice B)
- devices.toml: **6** perfis (piso: `moto-e5`)
- behavior-switches.toml: **25** comutadores targetSdk 26→36 (PRF-07)

## Compatibilidade por módulo (checklist PARTE 0)

- Núcleo (0.0): parcial (CORE-01..06 M0; VM/DEX chegam em M1/M2)
- Todos os módulos não-M0: `NOT_IMPLEMENTED(module_id)` estruturado
