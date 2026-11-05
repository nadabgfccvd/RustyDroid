# Comprix v1.6.0: perfil do lançamento (2026-11-05)

Complemento de `docs/comprix-latency.md` (PR #553). Aqui fica a análise do
lançamento (`launch`, observe desligado), que está em ~640 ms contra a meta de 450 ms.

## Condições

- Binário: `rd` perfil `profiling` (LTO off, codegen-units=32, debug=0), 2 vCPU, ~2 GB.
  O perfil `release` foi morto por falta de memória (SIGKILL) durante a compilação do `rd_vm`.
- Telemetria: `RD_PHASES=1 RD_LAUNCH_TRACE=1` (stderr do `rd mcp`).
- Profiler: `perf record -F 2000 -g` (linux-perf 6.12, `perf_event_paranoid=2`).
  O binário mantém símbolos (`strip = "none"` no perfil `profiling`).

## Telemetria por fase (uma execução)

| fase | ms | chamadas |
|---|---:|---:|
| `install` (leitura do DEX) | 40,7 | — |
| `launch_app_cold` (total) | 644,6 | — |
| intrínsecos (inclusivo) | 516,0 | 120.957 |
| `<clinit>` | 71,4 | 1.423 |
| resolução de método/classe | 16,0 | 62.352 |
| strings | 3,6 | 23.526 |

Fuel consumido: 1.394.701.

## Intrínsecos: inclusivo × exclusivo por classe

A telemetria v3 (`LaunchTrace::intrinsic_by_class`) separa o tempo **inclusivo**
(inclui intrínsecos aninhados) do **exclusivo** (sem os intrínsecos filhos diretos).
O exclusivo ainda inclui o trabalho guest que o intrínseco executa (interpretação,
`<clinit>`, resolução), porque esses contadores são outra categoria.

| classe declarante | exclusivo ms | inclusivo ms | chamadas |
|---|---:|---:|---:|
| `Ljava/util/concurrent/Executor;` | 238,4 | 301,7 | 25 |
| `Ljava/lang/Object;` | 25,1 | 25,1 | 7.622 |
| `LU3/i;` | 18,1 | 18,1 | 24.045 |
| `Ljava/util/ArrayList;` | 10,8 | 10,9 | 3.749 |
| `Landroid/database/Cursor;` | 10,2 | 10,2 | 2.549 |
| `Ljava/util/concurrent/atomic/AtomicReferenceFieldUpdater;` | 10,1 | 10,1 | 3.467 |

Leitura: quase metade do intrínseco está em `Executor` (25 chamadas). Ele roda os
runnables postados, então o exclusivo dele é o trabalho guest desses runnables
(primeira composição e o caminho do Room). Não é custo de despacho de host.
Os outros intrínsecos somam poucos ms cada.

## Perfil de CPU (`perf report --comm rd`, plano)

O perfil é **plano**: nenhum símbolo passa de ~4,5% das amostras. Os primeiros:

| símbolo | % |
|---|---:|
| `interp::exec_frame` | 4,43 |
| `miniz_oxide::inflate` (leitura do APK) | 3,27 |
| `interp::exec_frame_regs::exec_leaf_insn` | 2,98 |
| `malloc` | 2,18 |
| `intrinsics::instance_intrinsic_dispatch_full` | 1,96 |
| `FxHash` (`hash_one`) | 1,89 |
| `free` (cfree) | 1,81 |
| `Classpath::is_subtype` | 1,67 |
| `hashbrown` `Equivalent<String>` | 1,67 |
| `jsqlite::unb64` | 1,52 |
| `intrinsics::call_static_intrinsic_rooted` | 1,38 |
| `interp::resolve_or_run_host_invoke` | 1,31 |
| `Classpath::field_ref_shared` | 1,09 |
| `Engine::call` | 1,02 |
| `Classpath::method_ref` | 0,94 |
| `Heap::gc_host_enter` | 0,94 |

Interpretação: o custo está espalhado pelo despacho e pelas buscas por chave `String`
(hash, comparação, `malloc`/`free`, `is_subtype`, `method_ref`, `field_ref`) em
cada instrução ou chamada, e não em um único ponto. Não há um gargalo isolado para
corrigir com um patch pequeno. Com mais 30% de corte necessário (de 640 para 450 ms),
é preciso trabalho estrutural no interpretador.

## Próximos passos (ordem sugerida)

1. Eliminar `String` nas chaves de lookup do caminho quente: resolver `method_ref`/
   `field_ref`/`is_subtype` uma vez por site de invoke (cache por índice, não por nome).
2. Remover alocações por chamada de intrínseco (`instance_intrinsic_dispatch_full` e a
   cadeia `if class == ...` de `call_static_intrinsic_rooted`).
3. Inspecionar o que o `Executor` executa no lançamento (runnables postados): esse
   exclusivo de 238 ms é o maior bloco isolado.
4. Medir de novo com `perf record --call-graph dwarf` para ter a visão inclusiva por função.
5. Medir com perfil `release` numa máquina com mais de 2 GB (não é possível nesta sandbox).

Nenhuma destas mudanças foi aplicada. Esta página só registra o perfil e a telemetria.

## Perfil do golden completo e A/B do GC (2026-11-05)

- `perf` do golden inteiro (thread da VM `rd-main`, 9.176 amostras): ainda plano.
  Os maiores: `exec_frame` 6,7%, `exec_leaf_insn` 5,1%, `instance_intrinsic_dispatch_full` 2,6%,
  `resolve_or_run_host_invoke` 2,3%, `malloc` 2,3%, `Classpath::is_subtype` 2,0%,
  `call_static_intrinsic_rooted` 2,0%, `free` 1,6%, `Classpath::field_ref_shared` 1,6%,
  `Heap::gc_host_enter` 1,5%.
- A/B com `RD_GC=off` (2 execuções por modo): **não** é mais rápido. Criar lista ~3,2 s com GC
  desligado contra ~2,6 s com GC ligado. O GC não é a alavanca de latência; não se deve desligá-lo.
- Nota: o `perf` mostra a VM na thread `rd-main`, não no processo `rd`; use `--comm rd-main`.

## Perfil do golden por categoria (2026-11-05, `perf record -F 1500 -g`)

Binário `target/profiling/rd` (`codegen-units=256`, LTO off). Golden passou (GOLDEN PASSOU).
Amostragem: `perf report --comm rd-main --no-children`, 787 símbolos, 98,7% do tempo.
Agrupamento aproximado por nome de símbolo (não por módulo; é uma leitura, não um inventário):

| Grupo | % |
|---|---|
| Laço do interpretador (`exec_frame`, `exec_leaf_insn`, `exec_frame_regs`) | 17,3 |
| Outros símbolos `rd_*` (inclui `engine::call`, `heap`, `dex`) | 17,0 |
| Símbolos sem nome (`0x…` em bibliotecas sem símbolo) | 11,5 |
| Intrínsecos e stubs de host (`instance_intrinsic_dispatch_full`, `call_static_intrinsic_rooted`, `android_stub_surfaces`) | 12,5 |
| Heap e GC (`gc_due_slow`, `gc_host_enter`, `class_of`, `get_field`) | 11,3 |
| Classpath (`is_subtype`, `field_ref_shared`, `method_ref`) | 7,7 |
| Hashing (`FxBuildHasher`, `RandomState`) | 5,0 |
| `malloc` / `free` / `cfree` (símbolos de libc) | ~4 |
| Demais | 1,8 |

Correção à leitura anterior: o grupo "malloc/free" de 14,6% estava inflado, porque também
contava funções Rust monomorfizadas com `alloc` no nome. O `malloc` da libc, sozinho, fica em torno de 4%.

Observações de código (cada uma ~1%, sem alavanca única):
- `Heap::gc_due` cai em `gc_due_slow` por instrução sempre que `gc_frames` está vazio, porque o
  caminho rápido exige `!gc_frames.is_empty()`.
- `Heap::gc_host_enter` roda por chamada de host e empilha um `Vec` e um corte em `existing_cutoffs`.

Conclusão: o custo é difuso. Não há um gargalo que, sozinho, leve as ferramentas MCP de ~1,2 s
para perto de 345 ms. Atacar isso exige mudanças estruturais no interpretador ou no despacho de
intrínsecos, e não ajustes pontuais.

Não reutilizar a gravação `perf` de `/tmp/perf_g2.data` anterior: a primeira tentativa desta janela
falhou porque `perf` não estava instalado após o restore (`apt-get install linux-perf` resolveu).

## Renderização da sheet "Compra do Mês" (verificação)

A suspeita de render congelado (screenshots com MD5 idêntico) não se confirmou. Com o fluxo
correto (Pular → Nova lista → chip "Compra do Mês" → "Criar e adicionar itens" → campo →
`type_text` com `replace`), a tela mostra o texto digitado ("2 kg de arroz") no campo "Adição
rápida". Os MD5 iguais vinham de um roteiro que não passava pelo chip, então a tela não mudava.

## Telemetria por passo (`RD_LAUNCH_TRACE`, 2026-11-05)

`RD_LAUNCH_TRACE=1` agora imprime, a cada chamada MCP, uma linha `[step <ferramenta>]` no stderr
com o relatório zerado no início da chamada. Antes, o relatório saía só no launch. `clinit_top`
lista as classes com maior `<clinit>` inclusivo (inclui o trabalho aninhado).

Golden com o binário novo (`/tmp/golden_stderr.txt` é gerado por uma cópia do roteiro que não
envia o stderr para `DEVNULL`):

| Passo | Intrínsecos (chamadas) | `clinit` | Observação |
|---|---|---|---|
| "Pular" (`tap_text`) | 725 ms / 340 mil | 1,9 ms / 144 | ~1,15 s no total; custo dominado por intrínsecos |
| "Criar e adicionar itens" | 1,98 s / 859 mil (`tap_text`) | 3,5 ms | intrínsecos domina |
| ENTER (`key`) | 1,37 s / 478 mil | **483 ms / 79** | `clinit` concentrado em duas classes |

Causa do `clinit` no ENTER (uma vez por processo, na primeira vez que o item é processado):

- `Lm2/A;` (catálogo de unidades): 240 ms inclusivo.
- `Lm2/n;` (catálogo de hortifruti, aninhado em `m2/A`): 227 ms inclusivo.
- Ambos são `<clinit>` com ~1.400 e ~3.900 linhas de smali, montados com `listOf`, `Pair`,
  `mapOf` e `Lm2/n;->c/e/a` (cerca de 300 entradas). O custo é execução de bytecode no
  interpretador (~2 M de instruções a ~250 ns), não uma operação cara isolada.

Decisões e limites:

- Mover esses `<clinit>` para o launch reduziria o passo ENTER, mas é uma troca de onde o custo
  cai na medição, e não uma redução de trabalho. Não foi feito; depende de decisão sobre a
  métrica.
- Acelerar `listOf`/`Pair`/`mapOf` com intrínsecos do stdlib do Kotlin: no `<clinit>` do `m2/n`,
  `listOf` e `Ls0/c;->L` são ~317 dos ~650 sites de invoke estáticos (contagem estática, não
  execução). Exige cuidado com a semântica das listas; o ganho real não foi medido.
- O ganho grande exige reduzir o custo por instrução do interpretador, o que segue difuso
  (ver a seção de categorias acima).

## Primeiro toque, primeiro frame e swipe (2026-11-05)

Medido com `RD_LAUNCH_TRACE=1` (um relatório por chamada e uma linha `[frame-phase]` por frame).

**`observe` não adiciona custo: revela o frame.** Sonda em duas partes (`install → launch → tap "Pular"`
sem `observe`, depois `observe` sozinho):

| Chamada | Tempo | Observação |
|---|---|---|
| `tap_text "Pular"` sem `observe` | 328 ms | fuel 2,72 M |
| `observe` (primeiro, tela nova) | 1.110 ms | fuel 5,24 M |
| `observe` sem mudança | 0,2 ms | nada a redesenhar |

O total (~1,44 s) é o mesmo com ou sem `observe`. O primeiro `observe` paga o frame que ficou pendente.

**Decomposição do primeiro frame** (`[frame-phase]`):

| Fase de `draw_frame` | Fuel | Tempo |
|---|---|---|
| `layout_roots` | 0,78 M | 167–198 ms |
| `advance_clock(16)` (drena o looper) | **4,84 M** | **989–1.123 ms** |
| `record_view` | 0,14 M | 28–34 ms |

**O que se sabe sobre o custo dentro de `advance_clock` (correção desta seção):** a leitura
inicial de que o trampoline do Compose (`LB0/k0`) era a causa **não se sustenta**. Esse runnable
aparece repetido com `quando 64 <= 64`, `96 <= 96` etc., mas limitar as execuções dele no mesmo
instante virtual **não mudou o fuel** (4.837.700 sem limite; 4.838.098 com limite 1; 4.838.200 com
limite 3). O experimento não foi commitado.

Medido com `[adv]` (telemetria por tick de `advance_clock`, `RD_LAUNCH_TRACE=1`):

| Tick (ms virtuais) | Trabalho | Fuel | Tempo |
|---|---|---|---|
| 48 | drenagem | 7 mil | 3 ms |
| 48 | traversal | 11 mil | 3 ms |
| **64** | **drenagem** | **4,46 M** | **900 ms** |
| 64 | traversal | 369 mil | 84 ms |
| 96 | drenagem | 963 mil | 210 ms |
| 96 | traversal | 356 mil | 79 ms |

O custo está na **drenagem de t=64**, não no traversal. Com `RD_TRACE_METHOD=/` nessa janela há
~372 mil chamadas de método. O código do app é uma fração mínima (~50 chamadas de
`ConfiguracoesApp`, `Categoria`, `Nutriente`). O resto é runtime de biblioteca ofuscado:
`LU3/i` (assinaturas de checagem de null do Kotlin, provavelmente `kotlin.jvm.internal.Intrinsics`),
`LP/C0`, `LU/m` e `LA0/v` (não identificados com certeza). A drenagem é a retomada de um
continuation do kotlinx.coroutines (`Lj4/h`, `DispatchedContinuation`) no looper principal.

Conclusão: o primeiro frame é **composição de UI rodando interpretada**, com muitas chamadas
pequenas, e não um gargalo único do app. Reduzi-lo exige diminuir o custo por chamada no
interpretador ou evitar essa composição no primeiro frame. Nenhuma das duas foi feita.

**Swipe na lista** (uma lista com 1 item, sonda `/tmp/swipe_probe.py`):

| Gesto | Sem `observe` | Com `observe` |
|---|---|---|
| vertical (1000→400) | 295 ms | 525 ms |
| horizontal (900 → 60) | 68 ms | 220 ms |

Não reproduz os ~1,5 s relatados. A hipótese é que o número relatado veio de uma lista com mais itens,
ou de um frame pendente. Não investigado além disso.

**Ambiente:** a segunda tentativa de build do `rd_vm` foi morta pelo OOM do kernel (`rustc` com
~1,47 GB de RSS em 2 GB de RAM). Um swapfile de 3 GB resolveu. Não é mudança de código; relevante
para reproduzir medições.

## Otimização por chamada: resultados negativos (2026-11-05, rodada de 17:00 UTC)

Medição com `scripts/comprix_latency.py --runs 3` (mediana), mesmo APK v1.6.0,
binário `profiling` (`codegen-units=256`, `lto=off`):

| chamada | base (`8e7b053`) | `RD_FW_DEBUG` lido uma vez | meta |
|---|---:|---:|---:|
| launch (observe off) | 509,7 ms | 519,7 ms | 450 ms |
| launch (observe on) | 767,8 ms | 773,0 ms | 345 ms |
| tap "Pular" | 1035,0 ms | 1046,8 ms | 345 ms |
| tap "Nova lista" | 467,9 ms | 496,2 ms | 345 ms |
| swipe | 1292,2 ms | 1255,5 ms | 345 ms |

- **Hipótese refutada:** `call_host_instance_rooted` fazia `env::var("RD_FW_DEBUG")`
  a cada despacho de host (um por intrínseco de instância). Trocar por um
  `OnceLock` não reduziu o tempo dentro da faixa de ruído (±5%). Mudança não
  mesclada; o custo de `env::var` com poucas variáveis é baixo.
- **Perfil de CPU atual** (`perf record -F 999 -g`, um run do golden de
  latência, filtrado por `--comm rd-main`): o custo é espalhado. Maiores
  símbolos de tempo próprio: `exec_frame` 7,0%; `exec_leaf_insn` 5,7%;
  `instance_intrinsic_dispatch_full` 2,2%; `resolve_or_run_host_invoke` 2,1%;
  `malloc` 2,1%; `Engine::call` 1,9%; `Classpath::is_subtype` 1,9%; `cfree` 1,9%;
  hashing de `String` (hashbrown) 1,7%. Nenhum item acima de 7%.
- **Conclusão:** a diferença para as metas (lançamento ~510 → 450 ms;
  ferramentas ~470–1290 → 345 ms) é de 1,1× a 3,7×. Não se fecha com trocas
  locais de 1–2%. Caminhos com algum ganho possível exigem mudança de desenho
  (menos chamadas do runtime de UI interpretado, menos alocação por invocação,
  ou cache de resolução de invoke por índice em vez de `String`), e cada um
  precisa de A/B com 3 rodadas antes de merge.
