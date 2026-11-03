# ADR 024 — Inventário do estado thread-local do host e caminho para engines concorrentes

- **Status:** proposto (aceito quando mergeado, precedente ADR 008)
- **Data:** 2026-11-03 · **Escopo:** `crates/rd-vm/src/{android_spannable,jrandom,jregex,android_uri,xml_res,jannotations,jio_chars,juuid,jfile,intrinsics}.rs`, `HostState`
- **Complementa:** ADR 017 (seção Follow-up). Não substitui a ADR 017.
- **Relacionados:** ADR 021 (layering, etapa 1 = sessões com roteamento de entrada); ADR 023 (múltiplas sessões por processo, item 6: estado global do host precisa ser por engine).
- **Ordem:** ADR antes de código (CONTRIBUTING). Este PR é só documentação. A migração de código é um PR separado, depois da aceitação.

## Contexto

A ADR 023 (aceita) permite N sessões no mesmo processo, cada uma com seu `Engine`. A ADR 017 ainda declara a premissa "um `Engine` vivo por vez por thread", e o próprio ADR 017 diz que essa premissa cai só depois de migrar as tabelas para `HostState`. Sem essa migração, dois engines vivos na mesma thread leem e escrevem as mesmas tabelas indexadas por `ObjRef`.

Inventário feito no código em 2026-11-03 (`grep thread_local!` e `static` em `rd-vm`):

| Módulo | Estado | Chave | Tipo de estado | Reset hoje |
|---|---|---|---|---|
| `android_spannable.rs` | `STATES` | `ObjRef` | estado por objeto | GC + `reset_host_thread_state` |
| `jrandom.rs` | `SEEDS` | `ObjRef` | semente por `Random` | GC + reset |
| `jrandom.rs` | `UNIQ` | — (contador) | uniquifier de `java.util.Random` | reset |
| `jregex.rs` | `CACHE` | pattern + flags | cache de regex compilada | **não** é resetado (chave sem `ObjRef`, seguro) |
| `jregex.rs` | `MATCHERS` | `ObjRef` | estado de `Matcher` | GC + reset |
| `android_uri.rs` | `BUILDERS` | `ObjRef` | `Uri.Builder` | GC + reset |
| `xml_res.rs` | `PARSERS` | `ObjRef` | `XmlPullParser` | GC + reset |
| `xml_res.rs` | `ARRAYS` | `ObjRef` | `TypedArray` | GC + reset |
| `xml_res.rs` | `PARSER_HOST_BYTES` | — (contador) | orçamento de bytes do host (ADR 015) | reset |
| `jannotations.rs` | `INSTANCES` | `ObjRef` | instâncias de anotação | GC + reset |
| `jio_chars.rs` | `WRITERS_ST`, `READERS_ST`, `BYTES_ST` | `ObjRef` | writers/readers/streams | GC + reset |
| `juuid.rs` | `RNG` | — | semente de `UUID.randomUUID` (`RD_UUID_SEED`) | reset |
| `jfile.rs` | `VFS` | — | sistema de arquivos virtual (prefs, SQLite, arquivos) | `reset_vfs` por sessão (ADR 017 item 4) |
| `intrinsics.rs` | `APP_STDOUT` (`static Mutex`) | — | destino de `System.out`/`err` (processo inteiro) | não resetado |

Resumo: **9 módulos** com `thread_local!` e **1 `static`** de processo. Dos 9 módulos, 8 têm tabelas indexadas por `ObjRef` ou contadores que o engine precisa possuir. `jregex::CACHE` é a única tabela thread-local sem `ObjRef` que pode continuar compartilhada.

## Opções

**A. Migrar as tabelas para dentro de `HostState` (escolhida).** Cada `Engine` carrega o próprio `HostState`. As tabelas `ObjRef` e os contadores saem do thread-local e viram campos. `APP_STDOUT` vira campo de `HostState`, ou fica no `HostOut` de cada sessão (ADR 023 item 1). O VFS passa a ser estado de sessão, guardado no `HostState` da sessão. O cache de regex pode ficar global.
- Vantagem: o isolamento vale para qualquer número de engines, em qualquer thread. Elimina a classe de bug "entrada de um heap morto". Cobre o item 6 da ADR 023 sem depender de ordem de chamada.
- Custo: mudança mecânica em 9 módulos e nas chamadas de `reset_host_thread_state`. Precisa de teste de regressão de dois engines intercalados na mesma thread.

**B. Confinar cada engine a uma thread própria.** Os thread-locals ficariam corretos por construção.
- Rejeitada: o servidor MCP responde em uma thread. Uma thread por sessão aumenta o custo de RSS (ADR 015: o teto vale para o processo inteiro, e cada thread tem pilha). Também exige cruzar `Engine` entre threads a cada tool call, e `APP_STDOUT` continua global. Ou seja, o trabalho de B não elimina o item de `APP_STDOUT`, então não compensa.

**C. Trocar o estado na entrada de cada sessão (swap).** Antes de rodar uma sessão, guardar e restaurar as tabelas.
- Rejeitada como destino: é frágil. Uma tabela esquecida no swap vaza silenciosamente entre sessões, sem erro de compilação. Serve no máximo como ponte curta, e só com um teste que liste as tabelas.

## Decisão

1. A premissa "um `Engine` vivo por thread" fica **explicitamente revogada** para o caminho de sessões (ADR 023), mas o código continua valendo até a migração do item 3.
2. Caminho escolhido: **opção A**. Migrar as 8 tabelas com `ObjRef` ou contadores, mais `APP_STDOUT`, para `HostState`. `jregex::CACHE` fica como está.
3. Ordem de PRs (cada um com testes de regressão, RD Sentinel):
   1. Mover `APP_STDOUT` para `HostState`. Teste: dois engines com saídas separadas.
   2. Mover as tabelas de `ObjRef` (spannable, jrandom, jregex matchers, uri, xml, annotations, io). Teste: dois engines intercalados na mesma thread, cada um com objetos com o mesmo `ObjRef` numérico, sem vazamento.
   3. Mover `UNIQ`, `PARSER_HOST_BYTES` e a semente de UUID. Teste: determinismo (Lei 4 / PRF-04) não depende da história da thread.
   4. Mover o VFS para estado de sessão. Teste: duas sessões não veem arquivos uma da outra; `package_clear` continua apagando só o pacote.
4. Ao fim da etapa 3, `reset_host_thread_state` passa a resetar só o que ainda for thread-local (no caso, nada além do cache de regex, que não precisa de reset). A função pode ser removida, e a ADR 017 é atualizada para "superseded in part by ADR 024".
5. Nenhum `engine_id` roteado (ADR 023 item 2) entra antes da etapa 2. Essa é a ordem de dependência.

## Consequências

- Etapa 1 do layering (sessões) fica bloqueada até as etapas 1 a 4 acima. Estimativa: 4 PRs pequenos. Esta ADR não aceita a etapa 1 de código; ela só define o caminho.
- O teste `rd test` (M6) de isolamento entre cenários passa a valer também entre sessões, porque o estado sai do thread.
- O custo de `Engine::new` continua O(tamanho das tabelas), mas as tabelas agora são por engine, então o custo é o mesmo.

## Fora de escopo

- Threads reais por sessão e execução concorrente de executors (hoje executors rodam inline; ver compat_gaps).
- Compartilhamento de heap entre sessões (rejeitado na ADR 023).

## Estado de implementação

- **Passo 1 (APP_STDOUT por engine): feito.** O destino do System.out vive em `HostState` (`Engine::set_app_stdout`). O `static` virou padrão do processo (`set_default_app_stdout`), que só semeia `Engine::new`. O agente fixa o buffer da sessão em cada engine criado. Teste: `system_out_is_per_engine_not_process_global` (dois engines no mesmo thread, cada um com seu buffer).
- **Passo 2 (tabelas `ObjRef` para `HostState`): feito.** Cada módulo define um `Tables` (derive `Default`), e `HostState` tem um campo por módulo: `uri`, `spannable`, `random`, `regex`, `xml`, `io_chars` e `annotations`. Os `gc_keys`, `gc_entry` e `gc_purge` recebem `&Tables`, e o `gc.rs` passa `&self.fw.<campo>`. Os `reset_all` desses módulos saíram: um `Engine` novo nasce com tabelas vazias. Ficam thread-local, para os passos 3 e 4: `UNIQ` (jrandom), `PARSER_HOST_BYTES` (xml_res), o RNG de UUID (juuid) e o VFS (jfile). `CACHE` de regex fica como está (cache puro). `reset_host_thread_state` continua existindo só para esses restos. Testes de regressão: `spannable_state_belongs_to_its_engine` (dois engines intercalados, mesmo `ObjRef` 5, limpar um não afeta o outro) e `annotation_instances_belong_to_their_engine` (#230).
- Passos 3 e 4: pendentes. Depois do passo 3, `reset_host_thread_state` sai (ver item 4 da decisão).
