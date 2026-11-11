# ADR 038 — Gate de benchmark por contagem de instruções (não por tempo de parede)

- **Status:** implementado (Issue #629) · **Data:** 2026-11-10 · **Revisado:** 2026-11-11
- **Escopo:** `scripts/bench_ir.py` (novo), `crates/rd-dex/benches/ir-baseline.json` (novo),
  `crates/rd-dex/benches/parse_tables.rs`, `.github/workflows/bench.yml`,
  `.github/workflows/ci.yml`, `tests/scripts/bench_ir_test.sh` (novo)
- **Relacionados:** Issue #629, Issue #227, ADR 037, `RustyDroid.md` §3.2

> Este ADR foi **reescrito duas vezes** no mesmo dia. A primeira versão propunha
> mediana de 5 sem normalizar; a segunda, normalizar pela calibração. Ambas foram
> **medidas e reprovadas** com dados do próprio CI. O histórico das duas está em
> "Alternativas descartadas" — importa, porque as duas parecem razoáveis no papel e
> a única coisa que as distingue é a medição.

## Contexto

O §3.2 da especificação exige `bench.yml` com "fail em regressão > 10%". A
implementação usava `github-action-benchmark` com `alert-threshold: 110%` e
`fail-on-alert: true`, comparando cada execução com a **única** anterior guardada
em `gh-pages`.

Em 2026-11-10, o primeiro push em `main` depois do merge de #628 (que
comprovadamente **reduziu** o binário e não tocou em `rd-dex`) falhou com 3324 ns
contra 1884 ns. A mesma revisão medida localmente deu 3759/3676 ns com ICF e
3712/3716 ns sem ICF. Não havia regressão.

### O que a série histórica mostra

Os 191 resultados guardados vão de **1 736 a 17 370 ns** por iteração, com um
vazio entre 1,25× e 2× a mediana e um aglomerado estável em 5,1× (18 execuções
entre 16 777 e 17 370 ns, 3,5% de espalhamento). São classes de CPU, não ruído.

### O que três execuções consecutivas do mesmo código mostram

O dado que decidiu o desenho final. Três pushes seguidos, mesmo benchmark, mesma
máquina de código:

| run | commit | `calib_scan_20k` | `parse_method_ids_20k` | razão |
| --- | --- | --- | --- | --- |
| 731 | `583aa07` | 43 164 ns | 3 196 ns | 0,0740 |
| 732 | `e0439c0` | 68 506 ns | 3 234 ns | 0,0472 |
| 733 | `7584fd7` | 42 310 ns | 3 160 ns | 0,0747 |
| **espalhamento** | | **62%** | **2,3%** | **58%** |

A segunda versão deste ADR normalizava pelo `calib_scan_20k` justamente para
cancelar a máquina. O resultado foi o oposto do pretendido: **a calibração é a
grandeza ruidosa**. Normalizar deixou a métrica 5× pior que o valor bruto, e o
gate reprovou o push `7584fd7` de um commit que só mexia em documentação.

O espalhamento de 62% na calibração não é ruído de amostragem (o desvio dentro de
uma execução foi de 0,19%), é a mesma memória medida com largura de banda
diferente em VMs diferentes. Não existe estatística que conserte um mensurando
que depende da máquina.

## Decisão

**Medir instruções retiradas, não tempo.**

`parse_tables.rs` ganhou o modo `--count <alvo>`, que executa **uma** iteração de
um alvo e sai, para rodar sob `valgrind --tool=callgrind`. O gate
(`scripts/bench_ir.py`) compara a **diferença** entre o alvo e uma execução
`noop` contra o baseline versionado em `crates/rd-dex/benches/ir-baseline.json`.

Medido neste repositório:

| alvo | Ir (3 rebuilds) |
| --- | --- |
| `noop` | 878 378 |
| `parse_method_ids_20k` | 923 930 |
| `parse_field_ids_20k` | 923 958 |
| **delta de trabalho** | **45 552 / 45 580** |

Zero variância entre rebuilds do mesmo fonte. Em diretórios de build diferentes o
delta varia ~0,02%. O limiar de 10% do §3.2 fica a **500×** o ruído da métrica.

### Por que subtrair o `noop`

O custo fixo do processo é **95% da contagem** (878 378 de 923 930). Sem
subtrair, um limiar de 10% incidiria sobre o startup do runtime: um parse 20%
mais lento passaria como 0,9% do total. Com a subtração, o limiar incide sobre o
trabalho.

### Por que o baseline é versionado e não um histórico

A contagem é determinística, então não há o que acumular: um arquivo de baseline
no repositório basta. Quando uma mudança altera o trabalho legitimamente (ex.:
de-vectorização intencional), o PR roda `--write` e o diff de
`ir-baseline.json` aparece na revisão — que é exatamente o lugar onde uma
mudança de 10% em custo deve ser discutida.

### O wall-clock continua medido, mas é consultivo

`cargo bench` ainda imprime ns/iter e o `github-action-benchmark` ainda grava o
gráfico em `gh-pages`, com `fail-on-alert: false`. Serve para ver tendência entre
execuções da mesma classe de máquina; não serve para gate.

## Alternativas descartadas

- **Mediana de 5 sobre ns (proposta original da issue).** Rejogando os 191
  resultados reais: 21,3% de falso positivo e 36% das regressões de +19%
  perdidas. Janela 21 piora a detecção (14,4% FP, 38% perdidas), porque a linha de
  base passa a cobrir classes de máquina.
- **Normalizar por uma varredura de calibração.** Medida e reprovada: a
  calibração variou 62% entre execuções enquanto o alvo variou 2,3%.
- **Janela de pares (comparar só com resultados de valor próximo).** Backtest
  nos 191 resultados: 6,2% de falso positivo e apenas 67% de detecção em +15%.
- **Estimador `min`/`p10` das amostras.** Medido sob contenção: a razão por `min`
  variou 1,26–2,03 nas mesmas condições em que a média ficou em 1,83–2,14. O
  mínimo é sequestrado por uma única amostra não preemptada.
- **Teto absoluto em ns.** Com a série real indo de 1 736 a 17 370 ns, qualquer
  teto que passe na máquina lenta não pega nada na rápida.
- **Runner self-hosted / de tamanho fixo.** Resolveria, mas é infraestrutura fora
  do repositório e não protege PRs de contribuidores.

## Consequências

- **O que a métrica deixa passar, declaradamente:** uma mudança que troca
  instruções por ciclos (pior IPC, mais cache miss) aparece como neutra ou
  melhoria. Para "o parse ficou mais lento", instruções é o proxy padrão (é o que
  o `iai-callgrind` usa); para "ficou mais lento *nesta CPU específica*", não
  serve. O ns continua publicado como sinal secundário.
- O job depende de `valgrind`, que já vem na imagem do runner; há um
  `command -v valgrind || apt-get install` defensivo.
- `scripts/bench_gate.py` e seu teste, introduzidos no primeiro desenho, foram
  **removidos**: implementavam a normalização que a medição reprovou. Manter os
  dois seria manter um gate que não funciona.
- `tests/scripts/bench_ir_test.sh` cobre 27 casos: limite de 10% (+9,9% passa,
  +11% falha), subtração do `noop`, baseline ausente semeia em vez de falhar,
  baseline desatualizado é anunciado, e o baseline versionado continua cobrindo os
  alvos que o bench define.
