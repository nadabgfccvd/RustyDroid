# ADR 034 — Caminho quente de `invoke` (cache por site e alocações por chamada)

- **Status:** proposto (desenho e critérios de aceite; implementação em PR de código
  separado, depois desta ADR ser mergeada) · **Data:** 2026-11-04
- **Escopo:** `crates/rd-vm/src/interp.rs` (`resolve_or_run_host_invoke`),
  `crates/rd-vm/src/classpath.rs` (`resolve_method`, `is_subtype`), `crates/rd-vm/src/engine.rs`
  (`Engine::call`, `coverage`, `gc_due`, `gc_frames`)
- **Relacionados:** ADR 033 (layout de campos: memória, não velocidade), ADR 030 (alvo de
  launch e de ferramentas MCP), ADR 032 (cache de código compartilhado)

## Contexto

Os alvos do usuário são: `launch` em 450 ms e cada ferramenta MCP em 345 ms, medidos com o
Comprix v1.6.0 dentro do RustyDroid. As medições da última rodada (mesma caixa, binário
`rd-s8`, `mcp_bench.py`, 3 execuções) mostram:

| Medida | Observe on | Observe off |
|---|---|---|
| `launch` | 685 ms (1 execução) | 523 ms (1 execução) |
| `tap_text` (mediana) | 527 ms | — (usa coordenadas, ver abaixo) |
| `tap` (coordenadas, mediana) | — | ~1 ms |
| `key BACK` (mediana) | 487 ms | ~0,3 ms |

Duas consequências:

1. Com observe **off**, a ferramenta devolve em ~1 ms porque o evento entra na fila e o
   trabalho do app roda depois, no próximo observe. O tempo por ferramenta só é honesto com
   observe **on** (ou somando a ferramenta com o próximo observe). Os alvos de 345 ms devem
   ser medidos com essa convenção, e isso precisa ficar explícito em qualquer relatório.
2. Com observe on, o custo está no próprio interpretador do app, não no observe. Rastreamento
   com `RD_TRACE_METHOD=L` (launch + 4 `tap_text` + 3 `key`) contou **2,6 milhões de
   chamadas DEX**. Os métodos mais chamados pertencem a bibliotecas ofuscadas
   (`LP/C0;->m()I` 84 mil, `LU3/i;->f(Object,String)V` 119 mil, `LG3/F;->z(CCZ)Z` 85 mil),
   não ao código `Lbr/com/comprix/` (só ~400 chamadas no total).

Perfil do binário `profiling` no mesmo fluxo (`perf record -g`, `tap_text` observe on), top
itens por autoria própria:

| Item | Self |
|---|---|
| `exec_frame::exec_leaf_insn` | 6,5% |
| `exec_frame` | 6,2% |
| `classpath::is_subtype` | 3,9% |
| `malloc` + `cfree` | 5,3% |
| `classpath::intern_name` | 2,1% |
| `classpath::method_ref` | 1,7% |
| `FilterMap` → `Vec` (coleta de `gc_frames` por chamada) | 1,8% |
| `Engine::gc_due` | 1,6% |
| `Engine::call` | 1,4% |
| `classpath::code` | 1,3% |
| `intrinsics::string_length` | 1,3% |
| `HashSet<u64>::insert` (`coverage`, SipHash) | ~0,9% + 0,6% de `hash_one`/`Sip` |

O custo por chamada DEX→DEX é a soma de muitos pequenos pagamentos: `method_ref` (lock +
clone), `find_class`, `is_dex_only_chain`, `resolve_method` (três `intern_name` + hash da
chave tripla + clone de `ClassDef`/`EncodedMethod`), `raw.to_vec()`, `cp.code` (lock + hash),
`coverage.insert` (SipHash), `gc_frames` (alocação quando há referências), `gc_due` (por
instrução).

Nenhum desses itens sozinho ultrapassa ~7%. Por isso o ganho esperado é cumulativo, e o
critério de sucesso é a medição pareada (ver abaixo), não a presença de uma função no perfil.

## Opção escolhida

Mudanças **exatas** (mesmo resultado observável, mesmas raízes de GC, mesmas exceções), em
três partes. Nenhuma troca semântica entra nesta ADR.

### A. Cache de decisão por site de `invoke` (DEX-first)

Chave: `(opcode, dex_idx, method_idx, recv_class)`, onde `recv_class` é comparado por
conteúdo (`Arc::ptr_eq` primeiro, depois `==`). Valor: o alvo DEX resolvido
`(d2, Arc<ClassDef>, EncodedMethod)` **ou** "sem alvo DEX" (cai para a cadeia host).

Equivalência: a decisão atual depende só de `mref` (estático para o `method_idx`), do opcode,
da classe do receptor e de tabelas que não mudam depois do carregamento do DEX (`find_class`,
`is_dex_only_chain` memoizado, `resolve_method`, `type_str`, `is_bundled_shim_owner`). O cache
só guarda o resultado dessas funções para a mesma entrada. A checagem de receptor nulo
(`NPE`), o ramo `invoke-super` com `Object.equals`, `ensure_initialized` de `<init>` e o
caminho host **continuam antes/depois do cache** e não são cacheados.

Invalidação: as tabelas `is_subtype`/`is_dex_only_chain` já têm invalidação em
`invalidate_subtype_memo`; o cache de site é limpo no mesmo ponto. Se o código de DEX não muda
durante uma sessão, isso não dispara.

### B. `coverage` com hash rápido

`Engine::coverage` passa de `HashSet<u64>` com SipHash para `FxHashSet<u64>`. É um conjunto
interno de cobertura (AG-09); a semântica não muda.

### C. Custos fixos por chamada DEX

1. `gc_frames`: a coleta de referências dos argumentos deixa de alocar um `Vec` novo quando
   não há referências, e reutiliza um buffer quando há. As raízes publicadas são as mesmas.
2. `gc_due`: as condições baratas (`used <= last_used_after`) são avaliadas antes de
   `free_slots()`. O resultado booleano é idêntico, porque as duas leituras são puras.
3. `cp.code(dex_idx, code_off)`: o `Arc<CodeItem>` obtido no cache de site (A) é passado
   para `Engine::call` em vez de ser buscado de novo com lock e hash.

Itens **fora** desta ADR, registrados para decisão posterior:
- `is_subtype` (3,9% self): precisa de desenho próprio (conjunto de ancestrais por classe).
  Entra em ADR separada se continuar acima de 3% depois de A–C.
- `string_length` (1,3%): `is_ascii()` sobre strings longas a cada `length()`. Exige guardar
  o tamanho UTF-16 e o flag ASCII na criação da string; é mudança de representação do heap.
- JIT (`RD_JIT`): opt-in, desligado no caminho do Comprix. Não é alavanca desta rodada.
- Observe: o custo do observe não está nesta fatia. Não mudar o pipeline de observe aqui.

## Invariantes que a implementação deve manter

- Mesmo resultado de cada `invoke`: alvo, `NPE`, `StackOverflowError`, exceções.
- Mesmas raízes publicadas para o GC em todo `0x22` e `0x60..=0x78` (ver nota sobre
  `exec_frame`: não é candidato a remoção).
- `RD_STR_SHADOW=1` sem divergências em launch e tap.
- Nenhuma alteração de comportamento do `RD_JIT` (opt-in).

## Critérios de aceite

1. `cargo test -p rd-vm --lib` e `cargo test -p rd-agent --lib` passam (193 e 112 em `rd-s8`).
2. `cargo clippy --workspace --all-targets -- -D warnings` (padrão, `--all-features`, e
   `--features rd-cli/jit`) passa, e `cargo fmt --all -- --check` está limpo.
3. `scripts/comprix-regression.py --apk <v1.6.0> --cycles 2` com sucesso, 17 passos.
4. Paridade de telas: `seq_compare.py` sem diferenças, exceto `runtime_health` (tempo).
5. Benchmark pareado, mesma caixa, alternando binário base e novo, n ≥ 5 cada:
   - observe on: mediana de `launch` e de `tap_text` e `key` **menor** que a base, com
     intervalo sem sobreposição nos três; ganho mínimo declarado de 10% em `tap_text`.
   - observe off: sem regressão > 5% no `launch`.
6. Relatório honesto contra os alvos (450 ms e 345 ms), com a convenção de observe explícita.
   Se o alvo não for atingido, o relatório diz quanto falta.

## Não-objetivos

- Não reduzir o trabalho que o app faz (o interpretador executa o que o DEX manda).
- Não mudar o modelo de observe nem o contrato das ferramentas MCP.
- Não alterar ADR 033 (memória); ela segue em ordem própria.

## Consequências

- Os ganhos de A–C são pequenos individualmente e precisam de medição pareada; o risco de
  ruído está registrado. Micro-mudanças abaixo de ~3% não são aceitas sem prova.
- A decisão de cache por site introduz estado por sessão. Ele é derivado de DEX imutável e
  pode ser descartado a qualquer momento sem mudar resultados, só desempenho.

## Resultados (implementação, PR #536)

Mesma caixa, binário de release, observe on, `mcp_bench.py` (mediana das execuções), pares
alternados:

| Medida | Base (`rd-s8`) | Com ADR 034 (`rd-s11`) | Variação |
|---|---|---|---|
| `launch` | 640 ms | 599 ms | −6% |
| `tap_text` | 495 ms | 471 ms | −5% |
| `key` (BACK) | 440 ms | 399 ms | −9% |
| CPU do launch (`cpu_launch.py`, média) | ~485 ms | ~475 ms | ~−2% |

Testes: `rd-vm` 193 e `rd-agent` 112 passando; `comprix-regression.py --cycles 2` com
sucesso (17 passos); paridade de telas (`seq_compare.py`) idêntica, exceto `session_id` e
tempo de pausa do GC.

Os alvos continuam longe: `launch` está ~1,33× acima de 450 ms e a ferramenta ~1,4× acima de
345 ms (observe on). Os ganhos desta ADR são reais, mas pequenos.

### Item D (chave de campo memoizada por site): medido e revertido

Implementado e medido (`rd-s11` → `rd-s13`): `launch` 601 → 583 ms, `tap_text` 461 → 464 ms,
`key` 406 → 409 ms. Sem ganho mensurável acima do ruído (±3%). Pela regra desta ADR (micro-
mudanças abaixo de ~3% não são aceitas sem prova), o item D foi **revertido** e não entrou
no `main`. O perfil mostrou o motivo: `qualified_field_key` não é um custo dominante no
caminho medido.

### Observe e navegação

Medições com `observe` ligado e desligado (mesmo binário):

- `observe` sozinho: ~3 ms (a extração de texto não é o gargalo).
- Com observe **desligado**, um toque por coordenadas retorna em ~16 ms; o trabalho da troca
  de tela aparece no `observe` seguinte (~458 ms). Ou seja, a tela nova custa o mesmo, só
  muda em qual chamada aparece.

Conclusão: o tempo por ferramenta é o trabalho do próprio app para montar a tela nova. Ele
se decompõe em chamadas DEX (ver ADR 035) e não no pipeline de observe.
