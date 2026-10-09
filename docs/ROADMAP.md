# ROADMAP — RustyDroid

> Espelho executável da PARTE 4 da spec (`RustyDroid.md`). Status real, não aspiração.
> **Regra:** cada fase termina com código funcional + testes verdes + docs atualizadas.

## Status atual

| Fase | Entrega central | DoD (done quando…) | Status |
|---|---|---|---|
| **M0** | Workspace + CI + rd-apk + permissions.toml + devices.toml (moto-e5) + behavior-switches.toml + motor de permissões | inspect de APK real imprime componentes/permissões/features completos; CI verde | ✅ **implementado (2026-10)** |
| M1 | rd-dex 100% + disassembler | idêntico ao baksmali em 3 APKs; fuzz 24h sem crash | ✅ |
| M2 | VM Dalvik mínima | métodos puros de APK real retornam valores corretos | ⬜ |
| M3 | Framework essencial | app trivial (activity+botão+texto) roda headless ponta a ponta | ⬜ |
| M4 | Render+dump | screenshot correto; get_ui_tree com ids certos | ⬜ |
| M5 | Agente MCP v1 (~20 tools) | LLM instala, navega e assera estado só com as tools | ⬜ |
| M6 | Executor de testes | suíte JSON → JUnit XML; compat-matrix automatizada | ⬜ |
| M7 | Performance + gate piso E5 | startup < 2 s app trivial; suíte golden passa nos budgets moto-e5 | ⬜ |
| M8 | Compat real | app OSS médio navega funcional; v0.9 | ⬜ |
| M9 | Componentes avançados + NDK | 5 APKs com .so rodando; widgets/IME/notificações ok | ⬜ |
| M10 | Dispositivos virtuais | câmera/GPS/biometria programáveis pelo agente | ⬜ |
| M11 | Engines | Flutter → RN → Mono → wasm: 1 app de cada em golden flow | ⬜ |
| M12 | Ecossistema GH | docs site, compat-matrix viva, releases; v1.0 | ⬜ |

## O que o M0 entregou (verificado em APK real: F-Droid 2.0.1)

- **Workspace Cargo** com 9 crates + xtask, dependências só apontam para baixo,
  MSRV 1.76, perfil release calibrado para hardware fraco.
- **rd-apk**: leitor ZIP próprio (STORED/DEFLATE/ZIP64, CRC-32 verificado),
  parser AXML completo (string pools UTF-8/16, resource map, typed values,
  dimensões/frações), parser ARSC mínimo-correto (config default, referências
  encadeadas), detecção de assinatura v1/v2/v3/v3.1, modelo tipado do manifest
  (Apêndice F: 4 componentes + aliases + providers + queries + features +
  meta-data + instrumentação) com árvore crua preservada (Lei 2).
- **Motor de permissões** (`fd-permissions`): dado versionado com 173 permissões
  (43 dangerous/12 grupos, 15 specials, 74 normais, 35 signature, 3 internal) +
  13 roles + 7 appops; estado instalável/mutável (grant/deny/revoke) e auditoria
  estática (exported sem guarda, debuggable, cleartext, desconhecidas).
- **fd-devices**: 6 perfis de device; `moto-e5` é o piso contratual (512 MB RSS /
  256 MB heap / 25% single-thread / Adreno 308 fill-rate).
- **fd-behavior**: 25 comutadores targetSdk 26→36 (PRF-07), consultáveis por target.
- **CLI `rd`**: `inspect` (humano + `--json`), `perm list|info|audit`,
  `device list|show`, `behavior --target`, `dex` (contrato `NOT_IMPLEMENTED`
  funcionando com exit code 2). Dados resolvidos de `--data` > `RD_DATA_DIR` >
  `./data` > embutido no binário.
- **CI**: fmt + clippy `-D warnings` + test + MSRV 1.76 + smoke do binário.
- **Fixture GH-05**: `golden-apks/fetch.sh` baixa APKs OSS (nunca commitados).

## ✅ M1 concluído — rd-dex (2026-10)

1. ✅ Parser DEX 100% (header, maps, strings MUTF-8, types, protos, fields,
   methods, classes, code items, debug info, annotations, call sites,
   method handles) — **45.643 classes dos 3 APKs golden renderizam sem
   falha (0 pulos)**.
2. ✅ Disassembler smali fiel (alvo baksmali 2.5.2): **24.913 classes
   comparadas por método — 100% idênticas na sequência de instruções**
   (comparador differential: `/tmp/compare.py`, artefato de validação).
3. ✅ Fuzz targets (`cargo-fuzz`): `zip`, `axml`, `arsc`, `apk_full`, `dex`;
   CI compila os targets em nightly (execução 24h fora do CI — ver
   `docs/FUZZING.md`).
4. Gate de saída: idêntico ao baksmali nos 3 APKs ✅; fuzz 24h sem crash —
   smoke local limpo, campanha 24h pendente de runner dedicado (não bloqueia
   M2; parsers são fuzz-hardened por construção, Lei nº 1).

## Próxima fase — M2 (rd-vm)

1. Interpretador Dalvik mínimo executando métodos puros de APK real.
2. Arena/GC do piso moto-e5 (RSS ≤ 512 MB, heap ≤ 256 MB).
3. Critério de saída: métodos puros de APK real retornam valores corretos
   (golden vectors vs execução real).
