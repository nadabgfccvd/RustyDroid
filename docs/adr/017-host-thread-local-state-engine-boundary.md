# ADR 017 — Estado host thread-local: fronteira de `Engine`, pacote e sessão

- **Status:** proposto (aceito quando o dono mergear, precedente ADR 008)
- **Data:** 2026-10-25 · **Issue:** #230 · **PR:** #256 (autor ArenaAgent, publicado pelo PR-Agent)
- **Escopo:** `crates/rd-vm/src/{lib.rs, jfile.rs, android_spannable.rs, jrandom.rs, jregex.rs, android_uri.rs, xml_res.rs, jannotations.rs, jio_chars.rs, juuid.rs, engine.rs}` e `crates/rd-agent/src/mcp.rs`
- **Relacionados:** Lei 4 (determinismo), PRF-04 (sequências determinísticas por Engine), M6 (isolamento de cenários do `rd test`)

## Contexto

Várias superfícies host do rd-vm guardam estado em tabelas thread-local
(macro `thread_local!` do std) **fora** do `Engine`. São elas:
- spannable, random, regex matchers, uri builders, xml parsers/arrays,
  annotations e io-chars, todas indexadas por `ObjRef`;
- o RNG de UUID;
- o VFS (`jfile`: arquivos, SQLite serializado, shared_prefs).

Essas tabelas eram purgadas só durante o GC. Três vazamentos resultavam disso (#230):

1. **Engine → Engine na mesma thread.** `Heap::new` recomeça a numeração de
   `ObjRef`, então um Engine novo (relaunch, próximo cenário de suíte)
   encontrava entradas de um heap morto sob refs que agora apontam para outros
   objetos. Exemplo: um objeto arbitrário sendo despachado como instância de anotação.
2. **Pacote → pacote.** `package_clear` e `package_remove` não apagavam o
   armazenamento privado do pacote no VFS.
3. **Sessão → sessão.** Um `AgentSession` novo herdava o VFS inteiro do anterior.

## Decisão

1. **Premissa explícita: um `Engine` vivo por vez por thread.** É o padrão de
   todas as superfícies atuais (MCP, CLI, executor de suítes). Uma superfície que
   quiser Engines concorrentes na mesma thread precisa antes migrar essas tabelas
   para dentro de `HostState` (ver "Follow-up").
2. **Fronteira de Engine.** `rd_vm::reset_host_thread_state()` esvazia as sete
   tabelas indexadas por `ObjRef`, devolve o RNG de UUID à semente
   (`RD_UUID_SEED`) e reinicia o uniquifier de `java.util.Random`. É chamada em
   `Engine::new` e quando o MCP descarta o engine ativo. O cache de regex
   compiladas fica: a chave é pattern+flags, sem `ObjRef`.
3. **Fronteira de pacote.** `rd_vm::clear_package_data(pkg)` apaga
   `/data/user/0/<pkg>`, `/data/data/<pkg>` e
   `/storage/emulated/0/Android/data/<pkg>`, respeitando a fronteira de segmento
   (`com.example.a` não apaga `com.example.ab`), e fecha os descritores abertos
   nesses caminhos. Alimenta `package_clear` e `package_remove`.
4. **Fronteira de sessão.** `rd_vm::reset_vfs()` zera o VFS. É chamada por
   `AgentSession::new`: sessão nova = dispositivo limpo.
5. **O VFS NÃO é resetado entre Engines da mesma sessão.** Dados persistidos
   sobrevivem a um relaunch, como no Android.
6. **Sem `Drop` em `Engine`.** Acessar um thread-local durante o teardown da
   thread pode entrar em panic, e um panic dentro de `drop` aborta o processo.
   Os pontos explícitos (`Engine::new` e o descarte no MCP) cobrem todos os fluxos.

## Consequências

- Determinismo (Lei 4 / PRF-04): sequências de UUID e de `Random` voltam a
  depender só do Engine, não do histórico da thread.
- Isolamento real de cenários no `rd test` (M6), que antes só estava declarado.
- Custo: O(tamanho das tabelas) por `Engine::new`. Fora do caminho quente.
- Nenhum `thread_local!` novo foi introduzido. A mudança só dá fronteiras
  explícitas ao estado que já existia.

## Nota de supersessão (ADR 024, passo 4)

O item 4 (`reset_vfs()` na fronteira de sessão) foi superado: o VFS agora é um `VfsHandle` por sessão, guardado em `HostState.vfs`, e a fronteira de sessão é `VfsHandle::new()`. Os itens 5 e 6 seguem válidos.

## Follow-up

Migrar as tabelas indexadas por `ObjRef` para dentro de `HostState`. Isso
elimina a premissa do item 1 e torna os resets dos itens 2 a 4 desnecessários
para essas tabelas. O VFS continua sendo estado de sessão.

## Decisão sobre o D03 do eval (2026-11-08): registrar, não migrar

O eval de 2026-11-08 marcou regressão em `D03-GLOBAL-HOST-STATE` (`thread_local_count` 1 → 3 e um
hit novo de global mutável), vinda de #534 (perf), não de uma mudança desta rodada. Avaliação por local:

| Local | Tipo | Decisão |
|---|---|---|
| `heap.rs` (`STRING_CLASS`) | cache puro de `Arc<str>` com o nome de `java/lang/String` | Mantido. Não guarda `ObjRef` nem dado de sessão. |
| `intrinsics.rs` (cache de unidades UTF-16) | cache de um texto longo, com chave pelo próprio conteúdo | Mantido. Sem `ObjRef`, então `reset_host_thread_state` não precisa limpá-lo. |
| `jregex.rs` (`CACHE`) | cache de regex compiladas por (pattern, flags) | Mantido, como já decidido na seção de Decisão, item 2. |
| `intrinsics.rs` (`DEFAULT_APP_STDOUT`) | `Mutex<HostOut>` global, semente para `Engine::new` | Mantido com a fronteira atual: o destino de cada engine vive em `HostState` (`set_app_stdout`), e o global só é lido em `Engine::new`. Ver follow-up. |

**Por que não corrigir agora**: os três `thread_local!` são caches sem estado de sessão, e trocá-los por
estado por engine só mudaria a contagem do eval, com custo de desempenho. O `DEFAULT_APP_STDOUT` só
semeia engines novas e não vaza entre sessões (ADR 024, passo 1). Migrá-lo para `VmConfig` exige mudar
o `rd-cli` e o teste do `rd-engine-registry`, e fica como follow-up.

**Consequência no eval**: o desconto de D03 continua aparecendo como regressão de 0,2 na nota. Isso é
aceito e registrado aqui, não escondido. Se o eval passar a contar `thread_local!` de cache puro, o
critério deve ser ajustado no scorer, e não o código.

**Follow-up**: mover `DEFAULT_APP_STDOUT` para `VmConfig` (o padrão passa a vir da config da engine), e
rever a contagem de `thread_local!` no eval para separar cache puro de estado de sessão.
