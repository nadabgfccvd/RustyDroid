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

**Causa dominante: `advance_clock` drena o looper e executa o trampoline do Compose 29 vezes no mesmo
instante virtual.** O runnable `LB0/k0` (trampoline do dispatcher de frames do Compose, roda como
mensagem do Looper) aparece em `[fw] drain` repetido com `quando 64 <= 64`, `96 <= 96` etc. Ele se
reagenda com `when = now`, sem avançar o vsync. Cada execução custa ~165 mil de fuel, ou seja, uma
composição inteira. O cap `MAX_FRAMES_PER_DRAIN = 8` do `choreographer.rs` cobre só os callbacks do
Choreographer; estes passam pelo Looper e não são limitados.

Duas leituras, nenhuma implementada:
- **Pacing:** o trampoline deveria reagendar para o próximo vsync (como no device). Alinha com o modelo
  de tempo de `advance_clock`, mas muda a ordem e o momento dos frames de **todos** os apps que usam
  Compose. Exige a suíte completa e revisão de telas.
- **Cap por drenagem:** limitar as execuções do trampoline por drenagem, como já é feito para os
  callbacks do Choreographer. Mais local, mas também altera o que está pronto quando `observe` retorna.

Por isso não foi feito sem decisão. Se o pacing for adotado, o primeiro frame cai de ~4,8 M para
algo próximo de uma composição (~0,2 M de fuel), o que levaria o "Pular" para perto de 0,5 s.
Essa estimativa não foi medida.

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
