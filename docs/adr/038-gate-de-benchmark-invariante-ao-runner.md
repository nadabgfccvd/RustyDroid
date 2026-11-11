# ADR 038 — Gate de benchmark invariante à classe de runner (calibração normalizada)

- **Status:** implementado (Issue #629) · **Data:** 2026-11-10
- **Escopo:** `scripts/bench_gate.py` (novo), `crates/rd-dex/benches/parse_tables.rs`,
  `.github/workflows/bench.yml`, `.github/workflows/ci.yml`,
  `tests/scripts/bench_gate_test.sh` (novo)
- **Relacionados:** Issue #629, Issue #227, ADR 037, `RustyDroid.md` §3.2

## Contexto

O §3.2 da especificação exige `bench.yml` com "fail em regressão > 10%". A implementação
era `github-action-benchmark` com `alert-threshold: 110%` e `fail-on-alert: true`, que
compara cada execução com a **única** execução anterior guardada em `gh-pages`.

Em 2026-11-10, o primeiro push em `main` depois do merge de #628 (que comprovadamente
**reduziu** o binário e não tocou em `rd-dex`) falhou no job `benches (rd-dex parse tables)`
com 3324 ns contra 1884 ns (1,76×) e 3340 contra 1815 (1,84×). A mesma revisão medida
localmente deu 3759/3676 ns com ICF e 3712/3716 ns sem ICF — menos de 2% de diferença.
Não havia regressão: o gate estava medindo a frota.

### O diagnóstico correto não é "ruído"

Os 188 resultados guardados formam uma distribuição **bimodal com um vazio**:

| faixa (múltiplo da mediana) | resultados |
| --- | --- |
| < 0,75× | 32 |
| 0,75–1,0× | 62 |
| 1,0–1,25× | 71 |
| 1,25–2× | **0** |
| 2–4× | 5 |
| ≥ 4× | 18 |

As 18 execuções da última faixa não são cauda de ruído: ficam todas entre 16 777 e
17 370 ns, ou seja **5,1× com 3,5% de espalhamento**. São classes de CPU diferentes, cada
uma internamente reprodutível a ~1%. O que varia 10× é a série, não a medição.

Isso invalida o remédio estatístico. Rejogando os 188 resultados reais com a proposta
original da Issue #629 (linha de base = mediana das últimas *N*, limiar de 10%):

| janela | falsos positivos | regressões reais de +19% perdidas |
| --- | --- | --- |
| 1 | 29,7% | 31% |
| 5 | 21,3% | 36% |
| 21 | 14,4% | 38% |

Alargar a janela reduz o falso positivo e **piora a detecção**: a linha de base passa a
cobrir classes de máquina, e uma regressão de 19% numa máquina rápida não chega a
1,19× a mediana global. Mediana não separa 10% de código de 500% de hardware.

## Decisão

### 1. Normalizar a métrica em vez de endurecer a estatística

O bench publica uma terceira medida, `calib_scan_20k`: uma varredura sequencial de 8 B por
entrada sobre o **mesmo** buffer de 160 KB, sem alocação, em
`crates/rd-dex/benches/parse_tables.rs`. Ela acompanha a máquina (largura de banda de
memória, IPC, escala de frequência) do mesmo jeito que o decode acompanha, então a razão
`alvo / calibração` é estável entre classes.

Validação empírica no sandbox, com contenção de CPU sintética para simular runners mais
lentos:

| condição | parse_method (ns) | calib (ns) | razão |
| --- | --- | --- | --- |
| ocioso | 3 897 | 64 399 | 0,0605 |
| 1 núcleo, 3 processos em spin | 7 575 | 117 885 | 0,0643 |
| 16 processos em spin (oversubscrito) | 32 683 | 525 178 | 0,0622 |

O tempo absoluto varia **8,39×**; a razão fica dentro de 0,0601–0,0661. Em execuções
repetidas na mesma máquina o espalhamento cai de 6,2% (ns bruto) para 3,0% (razão).

### 2. Medir os três alvos intercalados por amostra

`measure_all` visita calibração, `parse_method_ids` e `parse_field_ids` dentro de **cada**
amostra de 30. Medir um alvo inteiro antes do outro colocaria a calibração sempre na cauda
mais quente do processo — deriva térmica e turbo incindiriam sobre o numerador e o
denominador em momentos diferentes, enviesando exatamente a razão que o gate usa.

### 3. O gate aplica a regra do §3.2 à razão, sobre a mediana das últimas 5

`scripts/bench_gate.py` normaliza cada resultado guardado com a calibração **da mesma
execução**, e compara com a mediana das últimas 5 razões. Com ~3% de ruído na razão, o
erro padrão de uma mediana de 5 é ~1,7%, o que põe o limiar de 10% do §3.2 a ~6σ.
A regra de 10% continua sendo a regra; só o mensurando mudou.

`fail-on-alert` passa a `false` e `github-action-benchmark` fica só como **escritor do
histórico** (o alerta dela acima de 110% da execução anterior continua, mas é consultivo).

### 4. Semear em vez de adivinhar

Os 188 resultados existentes não têm coluna de calibração, e nenhuma transformação
retroativa é possível. O gate ignora linhas não calibradas e, com menos de `--min-samples`
(3) linhas calibradas, imprime `seeding` e **passa**. São ~3 pushes de carência depois do
merge, documentados, em vez de um gate que falha por não ter com o que comparar.

## Alternativas descartadas

- **Só mediana de 5, sem normalizar.** Medido acima: 21,3% de falso positivo e 36% de
  regressões de +19% perdidas. É o plano original da Issue #629 e ele não funciona.
- **Janela maior (21).** 14,4% de falso positivo e detecção pior. Mais história não
  resolve mistura de populações.
- **Teto absoluto.** Com a série real indo de 1 701 a 17 362 ns, qualquer teto que passe
  na máquina lenta não pega nada na rápida.
- **Linha de base por impressão digital da máquina** (`/proc/cpuinfo`). Precisa de estado
  auxiliar em `gh-pages` que o `github-action-benchmark` não escreve, e deixa a primeira
  execução de cada classe cega — o mesmo problema do seeding, com mais peças móveis.
- **`min` ou `p10` das amostras como estimador.** Medido: a razão por `min` variou
  1,26–2,03 sob as mesmas condições em que a razão por média ficou em 1,83–2,14. O mínimo
  é sequestrado por uma única amostra não preemptada e não é mais robusto aqui.

## Consequências

- **`calib_scan_20k` é contrato congelado.** Mudar a semântica da varredura muda o
  denominador e invalida o histórico calibrado; a função está marcada com esse aviso.
  Se um dia for inevitável, o gate entra em `seeding` de novo — é o comportamento correto.
- O histórico não calibrado permanece no arquivo e continua alimentando os gráficos do
  `gh-pages`; ele só não participa do gate.
- Aviso não fatal quando a própria calibração está >2,5× ou <0,4× a mediana recente: a
  comparação ainda é feita, mas o log diz que o runner está fora das classes conhecidas.
- `tests/scripts/bench_gate_test.sh` cobre os 22 casos que importam — inclusive "mesmo
  código numa máquina 5× mais lenta passa" e "regressão de +15% numa máquina 5× mais lenta
  falha" — e roda em PR via `scripts-tests` no `ci.yml`, sem toolchain Rust.
- O gate só julga em `push` para `main`; `workflow_dispatch` mede e reporta, como antes.
