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
