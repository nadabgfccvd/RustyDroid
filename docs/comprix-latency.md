# Latência MCP do Comprix v1.6.0: medição de base (2026-11-05)

Medição reproduzível com `scripts/comprix_latency.py` (um processo `rd mcp` por
sessão, cronometro do write da requisição até a resposta parseada).

**Metas do projeto:** lançamento 450 ms com observe desligado; cada ferramenta
MCP 345 ms com observe ligado.

## Condições da medição (desvios explícitos)

- Binário: `rd` com o perfil **`profiling`** (`CARGO_BUILD_JOBS=1`,
  `CARGO_PROFILE_PROFILING_LTO=off`, `CARGO_PROFILE_PROFILING_CODEGEN_UNITS=32`,
  `CARGO_PROFILE_PROFILING_DEBUG=0`). O perfil `release` (LTO thin, codegen-units=1)
  foi morto pelo kernel por falta de memória na sandbox de 2 GB (SIGKILL no
  `rd_vm`). Os números de release podem diferir; o perfil de release precisa ser
  medido em máquina com mais memória.
- Máquina: sandbox Linux, 2 vCPU, ~2 GB RAM. **Não é Moto E5.**
- APK: `comprix-v1.6.0-x86_64-release.apk` (x86_64, 23 MB).
- Fluxo: install → launch → observe → tap "Pular" → wait 1000 → tap "Nova lista" →
  swipe 800 ms → key BACK → screenshot → ui_tree. Launch com observe desligado em
  sessão própria.
- Três execuções; a tabela mostra a mediana e as amostras.

## Resultado (mediana de 3 execuções)

| chamada | mediana ms | amostras ms | meta ms | atinge |
|---|---:|---|---:|---|
| launch (observe desligado) | 642.4 | 736.9, 642.4, 621.0 | 450 | não |
| launch (observe ligado) | 1002.4 | 1002.4, 1113.2, 996.1 | 345 | não |
| install | 15.9 | 15.9, 17.4, 15.4 | 345 | sim |
| observe | 0.3 | 0.3, 0.2, 1.5 | 345 | sim |
| tap_text "Pular" | 1267.9 | 1237.6, 1267.9, 1350.0 | 345 | não |
| wait 1000 | 79.0 | 79.1, 79.0, 78.0 | 345 | sim |
| tap_text "Nova lista" | 557.6 | 557.6, 544.6, 607.9 | 345 | não |
| swipe 800 ms | 1513.9 | 1625.1, 1470.5, 1513.9 | 345 | não |
| key BACK | 338.9 | 405.0, 337.2, 338.9 | 345 | sim (no limite) |
| screenshot | 104.8 | 106.0, 103.4, 104.8 | 345 | sim |
| ui_tree | 0.5 | 0.5, 0.6, 0.4 | 345 | sim |

## Leitura

- **Não atingido:** lançamento (642 ms sem observe; 1002 ms com observe, 2,2× a
  meta de 450 ms), o primeiro toque de navegação (1,2–1,35 s), o toque em
  "Nova lista" (~560 ms), o swipe sobre a lista (~1,5 s).
- Fora da meta por pouco ou dentro: `wait`, `screenshot`, `observe`, `ui_tree`,
  `install`.
- A latência do swipe sobe de ~90 ms (tela de onboarding) para ~1,5 s depois da
  navegação para a tela de listas. O custo está no frame da lista (composição e
  registro de display list), não no gesto.
- Estes números são base, não aprovação. Nenhuma meta foi relaxada.
