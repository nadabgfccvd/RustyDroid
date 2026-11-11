# ADR 023 — Revisão do ADR 001: múltiplas sessões por processo (etapa 1 do layering)

- **Status:** aceito (decisão do dono, 2026-11-02, bloco 2: opção "sessões" da ADR 021, etapa 1)
- **Data:** 2026-11-02 · **Escopo:** `crates/rd-agent` (`mcp.rs`: `AgentSession`, tools), `crates/rd-cli` (`perf.rs`: orçamento de RSS), documentação
- **Revisa:** ADR 001 (sessão MCP "uma por processo"), na parte que fixa uma sessão por processo. O transporte JSON-RPC sobre stdio não muda.
- **Relacionados:** ADR 021 (layering, opção A); ADR 015 (teto de RSS do processo); ADR 017 (estado thread-local do host por engine); objetivos adicionais do dono em `docs/ROADMAP.md`.
- **Ordem:** este ADR é um PR só de documentação e deve ser mergeado ANTES de qualquer código de múltiplas sessões ou de camadas (regra do CONTRIBUTING: ADR antes de PR de código para decisões de arquitetura).

## Contexto

A ADR 001 define o servidor MCP do `rd-agent` com JSON-RPC 2.0 próprio e registra "uma sessão por processo" (`crates/rd-agent/src/mcp.rs`, doc de `AgentSession`). O mesmo arquivo documenta que a sessão tem **um app por vez**: `engine: Option<Engine>`, `engine_package`, `native_engine`, `activity`, `pending_snapshot`, `screen` e `ui_epoch` ficam no mesmo struct. O `HostOut` também é global no processo.

A ADR 021 pede dois APKs rodando juntos, um por cima do outro. A opção A (duas sessões com roteamento de entrada) foi escolhida pelo dono, na etapa 1: sessões separadas com roteamento de entrada; a composição visual vem depois. Essa etapa não cabe no modelo atual. Ela exige mais de um engine vivo no mesmo processo.

## Decisão

1. **Um processo pode hospedar N sessões de app.** Cada sessão tem o próprio `Engine`, namespace de classes, diretório de dados, engine de permissões, `HostOut` e estado de activity. Nada disso é compartilhado entre sessões. Isso mantém o isolamento de heap e de permissões que a ADR 021 descarta na opção B.
2. **Identificação por `engine_id`.** Cada sessão aberta recebe um `engine_id` estável. As tools `session_launch`, `session_list`, `session_tap` (e as que atuam sobre um app: `tap`, `swipe`, `input`, `screen`) recebem `engine_id` opcional. Sem `engine_id`, a tool atua sobre a sessão ativa, como hoje. Os resultados continuam carregando `session_id`.
3. **Roteamento de entrada fica fora dos engines.** Toques injetados numa sessão passam pelo roteador de entrada do dispositivo virtual (M10), e não por acesso direto ao estado de outra sessão. Uma sessão nunca lê a memória ou as classes de outra.
4. **Refactor antes de comportamento novo.** O estado de execução de `AgentSession` é extraído para `RunningApp` sem mudar comportamento. Os testes existentes de `mcp.rs` precisam continuar passando nessa etapa. Só depois entram as tools novas.
5. **Orçamento de memória.** O teto de RSS da ADR 015 vale para o **processo inteiro**, não por engine. Com N sessões, a soma das sessões precisa caber no teto. `rd perf` (`crates/rd-cli/src/perf.rs`) mede a soma. Cada sessão recebe um orçamento de `teto / N` e falha fechada se passar do seu limite. O limite de N é configurável e padrão 2, para o caso clicker + auto-clicker.
6. **Estado global do host.** Os estados thread-local do host (ADR 017) precisam ser por engine, ou ser trocados ao entrar em cada sessão. A troca de contexto precisa de teste de regressão antes de duas sessões rodarem em paralelo. A execução segue single-thread e determinística (decisão M3): as sessões alternam por fila, não por threads.

## Consequências

- O modelo "uma sessão por processo" deixa de valer. Clientes MCP que usam a sessão padrão continuam funcionando sem mudança, porque sem `engine_id` a tool usa a sessão ativa.
- Teste de regressão obrigatório (convenção do RD Sentinel, retirado pela ADR 010: mudança de VM e de sessão): duas sessões com estado isolado (uma sessão não vê classes, prefs nem `HostOut` da outra) e roteamento de toque para a sessão certa.
- A etapa 2 (composição visual com janela transparente, `rd-render` com duas camadas) não é coberta por esta ADR. Ela ganha ADR própria quando começar.
- Fora do escopo: threads por sessão, sessões em processos separados e compartilhamento de heap.

## Alternativas consideradas

- **Manter uma sessão por processo e abrir um processo por app.** Isolamento total, mas o roteamento de toque entre processos exige um canal novo e duplica o custo de RSS sem o orçamento compartilhado.
- **Um engine com dois apps no mesmo heap (ADR 021, opção B).** Rejeitada: quebra o isolamento de permissões e dados.

## Estado da implementação (etapa 1)

Atualizado na segunda rodada da etapa 1. O texto das decisões acima não muda. Os desvios e as lacunas ficam aqui, para que o registro seja fiel ao código.

- **Tools (desvio da decisão 2).** A implementação não criou `session_launch` nem `session_tap`. As tools são `session_open`, `session_list` e `session_close`. Todas as tools que atuam sobre um app (`launch`, `tap`, `tap_view`, `find_views`, `install`, `session_info` e as demais) aceitam `engine_id` opcional. Sem `engine_id`, a tool atua sobre a sessão padrão, que é a primeira e a que clientes atuais já usam. Assim a mesma capacidade vem sem duplicar a superfície. O catálogo MCP passa de 47 para 50 tools.
- **Limite de sessões (decisão 5, N).** `RD_MCP_MAX_SESSIONS`, padrão 2, limitado a 1..=8. Com o limite atingido, `session_open` retorna `SESSION_LIMIT`.
- **Refactor (decisão 4).** `AgentSession` foi separado em `RunningApp` antes do comportamento novo (PR #489).
- **Roteamento de entrada (decisão 3): roteador do hub, NÃO o M10.** O M10 não tem roteador de entrada (`crates/rd-agent/src/m10_tools.rs` e `crates/rd-vm/src/m10/` tratam sensores, câmera, conectividade e mídia). O roteador implementado é `InputRouter` em `crates/rd-agent/src/session_hub.rs`. Toda tool que injeta entrada (`tap`, `swipe`, `tap_view`, `tap_text`, `type_text`, `key` e `batch`) passa por ele antes de chegar ao engine da sessão de destino. Cada evento fica registrado com número de sequência, sessão de destino e se o app aceitou. `session_list` mostra `last_input_seq` por sessão. Eventos recusados ficam registrados como não aceitos. Um toque nunca chega ao engine de outra sessão.
  - **Lacuna:** a decisão 3 cita o M10. Isso não foi feito, porque o M10 não tem roteador. Se o dono quiser o roteador dentro do M10, é uma decisão de desenho separada.
  - **Lacuna:** `batch` é registrado como um evento só. Os passos internos não são registrados um a um.
- **Orçamento de memória (decisão 5).** Implementado como **orçamento de heap por sessão**, não como RSS por sessão. O teto é `ram_process_cap_mb` do perfil de piso de hardware (moto-e5, 1024 MB, ADR 015). Cada sessão recebe `min(ram_app_heap_mb, teto / N)` = `min(256 MB, 1024 MB / N)`. A soma dos orçamentos de N sessões nunca passa do teto (`session_heap_budget_bytes`, com teste). O orçamento vai para o `Heap` do engine no `launch`. Quem passa dele recebe `OutOfMemoryError` do VM, ou seja, falha fechada. `session_info` mostra `memory.heap_budget_mb`.
  - **Lacuna:** RSS por sessão não é medido. RSS é do processo inteiro. `rd perf` continua medindo o pico de RSS do processo, não o de cada sessão.
- **Estado global do host (decisão 6).** Auditoria: o único estado mutável de processo em `rd-vm` que a execução lê é `DEFAULT_APP_STDOUT`, que só semeia engines criados depois (ADR 024, passo 1). O único `thread_local!` é um cache puro de regex. A execução é síncrona, uma chamada por vez, sem threads por sessão.
- **Testes de regressão** (`crates/rd-agent/src/session_hub.rs`, módulo `session_hub_tests`, 11 testes):
  - isolamento de app instalado entre sessões;
  - toque roteado à sessão que lançou o app; a sessão padrão, sem app em execução, recusa o mesmo toque (`NOT_RUNNING`);
  - dois apps rodando ao mesmo tempo, com toques intercalados: o token de uma sessão não age na outra, e um toque na sessão A não muda a árvore da sessão B;
  - roteador de entrada: ordem, sessão de destino e toques recusados;
  - limite de sessões, fechamento, `session_list` (padrão primeiro);
  - orçamento de heap: divisão do teto, soma sempre dentro do teto, relatório em `session_info`, falha fechada com orçamento zero;
  - tools de sessão recusadas numa sessão avulsa.
