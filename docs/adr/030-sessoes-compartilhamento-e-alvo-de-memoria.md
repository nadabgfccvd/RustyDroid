# ADR 030 — Sessões simultâneas: custo marginal medido e caminho para o alvo

- **Status:** proposto; atualizado em 2026-11-04. Passo 1 da decisão (DEX/APK
  compartilhados) implementado e medido na ADR 031 (PR #528, `bedbb98`). Passos 2 e 3
  pendentes. A medição abaixo é a original, antes do compartilhamento · **Data:** 2026-11-04
- **Escopo:** `rd mcp` com N sessões (`SessionHub`, `crates/rd-agent/src/session_hub.rs`),
  `Classpath`/`Engine` em `rd-vm`, `Dex` em `rd-dex`
- **Relacionados:** ADR 023 (orçamento de heap por sessão), ADR 024 (sessões isoladas),
  ADR 028 (valores e nomes compactos), ADR 029 (composição do RSS)

## Contexto

O alvo de trabalho é "2 → 6 sessões simultâneas por 200 MB" (definição de trabalho:
sessões simultâneas por teto de 200 MB). A ADR 029 mostrou que a memória por sessão é
dominada por dados duplicados: DEX descomprimido, APK, código decodificado, índices e
objetos. Esta ADR mede quanto custa a segunda sessão no servidor MCP real e registra o
que falta para o alvo.

## Medição: custo marginal de uma sessão no `rd mcp`

Build debug, binário com a mudança da chave do `resolve_cache` (ADR 029, ação 0).
Driver: `rd mcp` por stdio, `session_open` para as sessões extras; cada sessão faz
`install` + `launch` do F-Droid + `screenshot`. Pico de RSS lido de `VmHWM` ao final.

| Cenário | Pico (`VmHWM`) |
|---|---|
| 1 sessão (default) | 175 744 KB |
| 2 sessões (mesmo APK) | 303 748 KB |
| Custo marginal da 2ª sessão | ≈ 128 000 KB (≈ 73% de uma sessão) |

Cada sessão, em separado, repete o trabalho que as outras já fizeram: o `install` lê e
parseia o APK; o `launch` infla e monta os DEX de novo (`dexes_of_apk` por engine) e
decodifica o código. A captura de tela das duas sessões tem o mesmo tamanho (39 082 bytes).

Referência de uma sessão fora do MCP (`rd app shot`, mesmo binário): 163 MB. O servidor
MCP soma cerca de 12 MB a mais por processo (hub, estado de sessão, buffers de log).

## Opções avaliadas

1. **Compartilhar DEX e APK entre sessões do mesmo arquivo.** Chave: caminho canônico,
   tamanho e mtime do APK. O `Classpath` guardaria `Arc<Dex>` (ou um tipo com `Deref`
   para `Dex`), e o `SessionHub` manteria o cache. Estimativa (não medida): 20 MB de DEX
   e até 12 MB de APK por sessão. Bloqueios conhecidos: `Dex` contém `OnceLock` e um
   cache de strings paginado; é preciso verificar `Send`/`Sync`. Há 56 usos de `.dexes`
   e 205 construções de `Classpath`/`Engine`; a API atual deve ficar, com construtores
   novos para o caminho compartilhado.
2. **Compartilhar o código decodificado** (`code_cache`) entre sessões. Os `CodeItem`
   são imutáveis; o cache hoje é por `Classpath`. Estimativa: até ~10 MB por sessão.
   Depende da opção 1 (a chave do cache é o `dex_idx`).
3. **Layout de campos por classe** (ADR 029, opção 4). Os campos de instância guardam o
   nome em cada entrada (`Vec<(Arc<str>, Value)>`). Um layout por classe, com um índice
   de nome para posição, eliminaria o nome por campo. Estimativa: 5 a 8 MB por sessão
   com muitos objetos. Os 393 pontos de acesso por nome podem manter a API.
4. **Reduzir o heap de objetos e as tabelas de instâncias** (`Vec<HeapObj>` com folga de
   capacidade). Depende da opção 3.

## Conta do alvo

- Custo marginal atual: ≈ 128 MB por sessão.
- Para 6 sessões em 200 MB, o custo marginal de cada sessão precisa ficar em ≈ 33 MB
  (200 MB menos o custo base, dividido pelas sessões adicionais).
- As opções 1 e 2 juntas dão uma redução estimada de 30 a 40 MB por sessão, levando o
  custo marginal para cerca de 90 MB. As opções 3 e 4 somam mais 5 a 15 MB. Mesmo com
  as quatro, o custo marginal fica acima de 60 MB.
- **Conclusão:** o alvo de 6 sessões em 200 MB não é atingível só com compartilhamento e
  layout. Exige uma redução grande do estado por sessão (heap de objetos e estado de
  execução), que hoje não está caracterizado.

## Decisão

Nenhuma mudança de compartilhamento nesta rodada. Próximos passos, em ordem:

**Estado (atualizado):**

- Passo 1: feito. ADR 031, PR #528 (`bedbb98`). Custo marginal da 2ª sessão caiu de
  127 884 KB para 80 912 KB (−36,7%); 1 sessão sem mudança; screenshots idênticas.
  Critério (≥ 25%) atendido. A verificação de `Send`/`Sync` passou (`Dex`, `Arc<Dex>`).
- Passo 2: pendente. Ainda não há caracterização do estado por sessão que não é DEX
  com duas sessões. Esse é o bloqueio atual para o alvo.
- Passo 3: pendente. A escolha entre o layout por classe e a redução do estado de
  execução depende do passo 2.
- Medição em release: pendente (ver seção abaixo).

**Plano original (histórico):**

1. ADR próprio para compartilhamento de DEX/APK entre sessões (opção 1), com a verificação
   de `Send`/`Sync` e um teste de duas sessões com o mesmo APK. Critério: custo marginal
   da 2ª sessão cai pelo menos 25% no mesmo driver.
2. Caracterizar o estado por sessão que não é DEX (heap de objetos, estado de execução)
   com o amostrador da ADR 029, rodando duas sessões.
3. Só então escolher entre o layout por classe (opção 3) e a redução do estado de execução.

## Medição em release: bloqueada nesta sandbox

Tentativas de build `--release -p rd-cli` (sandbox com 2 GB de RAM):

- perfil do repositório (thin LTO, `codegen-units=1`): `rustc` de `rd-vm` foi morto pelo
  OOM com ~1,4 GB de RSS (já na ADR 008);
- `lto=false`, `codegen-units=16`: mesmo OOM, mesmo tamanho de processo;
- `lto=false`, `codegen-units=256`: mesmo OOM.

O pico do `rustc` em `rd-vm` está antes da geração de código, então não depende do número
de unidades. O número de release continua desconhecido. Para medir, é preciso uma máquina
com ao menos 4 GB livres: `cargo build --release -p rd-cli` e depois `rd app shot` com
`VmHWM`, mesmo driver da seção de medição.

## Consequências

- O número honesto hoje é: uma sessão do `rd mcp` ocupa cerca de 176 MB de pico, e cada
  sessão extra custa cerca de 128 MB. Com o código atual, 200 MB comportam uma sessão.
- O driver de medição (`drive.py`) fica fora do repositório, em
  `/home/user/logs/mcp-2s/`, e pode ser reproduzido com `rd mcp` e as ferramentas
  `session_open`, `install`, `launch` e `screenshot`.
