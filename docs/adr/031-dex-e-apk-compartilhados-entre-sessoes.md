# ADR 031 — DEX e APK compartilhados entre sessões do `rd mcp`

- **Status:** aceito e implementado nesta PR (desenho revisado em 2026-11-04; medição abaixo)
  · **Data:** 2026-11-04
- **Escopo:** `crates/rd-agent/src/mcp.rs` (`dexes_of_apk`, launch, `install`),
  `crates/rd-agent/src/session_hub.rs` (`SessionHub`), `crates/rd-vm/src/classpath.rs`
  (`Classpath::dexes`), `crates/rd-vm/src/engine.rs` (`Engine::new`)
- **Relacionados:** ADR 023 (revisão do ADR 001: múltiplas sessões por processo), ADR 024 (inventário de estado thread-local),
  ADR 030 (custo marginal medido de uma sessão extra)

## Contexto

A ADR 030 mediu no `rd mcp`: 1 sessão com pico de 175 744 KB, 2 sessões com 303 748 KB.
A segunda sessão custa ≈128 MB, quase o custo de uma sessão inteira. Cada sessão faz
o próprio `install` (lê e parseia o APK) e cada `launch` chama `dexes_of_apk`, que infla
e monta os DEX de novo. Os bytes são idênticos entre sessões do mesmo arquivo.

## Fatos verificados no código

- `Dex` e `Arc<Dex>` são `Send + Sync` (verificado por asserção de compilação em
  2026-11-04). `Dex` tem `OnceLock` para o índice de classes e para as páginas de
  strings, então o compartilhamento não exige `unsafe` nem `Mutex`.
- `Dex` é `Clone`. Hoje, clonar um `Dex` copia os bytes: não economiza memória.
- `Classpath::dexes` é `Vec<Dex>` público. O código de produção que monta o `Classpath`
  fica em `engine.rs` (`Engine::new`) e em dois pontos do `rd-agent` e do `rd-cli`.
  Os demais construtores estão em módulos de teste.
- O `resources` do `launch` já é `Arc` (`set_resources_arc`), mas é recriado por sessão
  no `install`.

## Desenho proposto

1. **Chave de identidade do APK:** caminho canônico, tamanho e mtime. Mudou qualquer um,
   é outro APK e o cache não é reutilizado.
2. **Cache no `SessionHub`:** `HashMap<ChaveApk, Weak<SharedApk>>`, onde `SharedApk`
   guarda o `Apk`/`Zip` e os `Arc<Dex>` já montados. O `Weak` faz o conteúdo ser liberado
   quando a última sessão que o usa fecha.
3. **`Classpath` com `Arc<Dex>`:** o campo passa a ser `Vec<Arc<Dex>>`. Como `Arc<T>`
   tem `Deref<Target = T>`, os acessos `self.dexes[i].metodo()` e `dex.campo` continuam
   compilando. Novos construtores: `Classpath::new_shared(Vec<Arc<Dex>>)` e
   `Engine::new_shared(...)`. Os construtores atuais (`Vec<Dex>`) ficam, e embrulham em
   `Arc`. Os sites de teste (a maioria dos ≈50 usos de `Classpath::new`/`Engine::new` fora de comentários) não mudam.
4. **Código decodificado continua por sessão** neste passo (`code_cache` do `Classpath`).
   Compartilhá-lo é a opção 2 da ADR 030 e depende deste desenho.
5. **Sem mudança de semântica:** os `Dex` são imutáveis depois de montados. Qualquer
   código que precise de um `Dex` modificado deve construir um `Dex` próprio, fora do cache.

## Riscos

- Um teste ou caminho que dependa de `Arc::get_mut` sobre o `Dex` quebra quando o `Dex`
  está compartilhado. O compilador aponta os casos.
- O cache mantém o APK aberto enquanto houver sessão; `Weak` evita vazamento após o
  fechamento.
- Dois APKs com o mesmo caminho e tamanho mas conteúdo diferente: o mtime cobre o caso
  comum; a chave pode incluir também o CRC do diretório central, se necessário.

## Critérios de aceite

- Custo marginal da 2ª sessão no driver da ADR 030 cai pelo menos 25% (hoje ≈128 MB).
- Captura de tela das duas sessões idêntica à de uma sessão (`cmp`).
- Testes: `rd-agent` (incluindo `session_hub`), `rd-vm --lib`, `rd-vm` integração,
  golden VM e a suíte do workspace no CI.
- Teste novo: duas sessões com o mesmo APK não compartilham estado de `System.out`,
  disco virtual nem heap (isolamento da ADR 024 preservado).

## Consequências

- O ganho esperado é de 20 a 35 MB por sessão adicional (DEX e APK), não o alvo de 3×.
  A ADR 030 mostra que o alvo exige também reduzir o estado de objetos e de execução.
- A mudança é contida: alguns construtores novos, um campo com tipo novo, e o cache no
  hub. Nenhum formato de arquivo ou protocolo MCP muda.

## Implementação

- `rd-vm`: `Classpath::dexes` passa a ser `Vec<Arc<Dex>>`; novos `Classpath::new_shared` e
  `Engine::new_shared`. `Engine::new` e `Classpath::new` continuam com a mesma assinatura.
  Os testes que editam metadados de DEX usam `Arc::make_mut`, que não copia quando a
  referência é única.
- `rd-agent/src/apk_share.rs`: registro `ApkKey -> Weak<SharedApk>`. A chave é caminho
  canônico, tamanho e mtime (ns). `SharedApk` guarda o `Apk`, os `bundle_splits` e os DEX
  montados como `Weak<Dex>`.
- Os DEX ficam como `Weak` de propósito: quem os segura é o engine em execução. Sessões que
  rodam ao mesmo tempo compartilham; quando o último engine para, a memória volta, como antes.
- O APK continua preso ao `InstalledApp` até remover o pacote ou fechar a sessão (igual
  antes). O caminho de snapshot (ADR 009, opt-in) não usa o registro.
- Testes (`rd-agent`, `apk_share_tests.rs`): duas sessões no mesmo arquivo compartilham o
  `Arc<Apk>` e os `Arc<Dex>`; o registro solta a entrada quando a última sessão fecha;
  arquivo com outro mtime não é reusado; falha de montagem não fica em cache; sem `Arc`
  vivo, a próxima montagem é nova.

## Medição (debug, `rd mcp`, `VmHWM`, mesmo APK F-Droid, install + launch + screenshot)

| Binário | 1 sessão | 2 sessões | Custo marginal da 2ª |
|---|---|---|---|
| base (`858b635`, `rd-rc-s13`), 1ª rodada | 175 744 KB | 303 748 KB | 128 004 KB |
| base, 2ª rodada (pareada) | — | 303 628 KB | — |
| ADR 031 (`rd-shared-s14`) | 176 552 KB | 256 860 KB | 80 308 KB |
| ADR 031 final (`rd-shared2-s14`) | — | 257 464 KB | 80 912 KB (257 464 − 176 552) |

Redução do custo marginal da 2ª sessão: 36,7% (127 884 KB para 80 912 KB). A base usa 303 628 KB
(2 sessões, rodada pareada) menos 175 744 KB (1 sessão, 1ª rodada). A 1ª sessão do binário final
não foi medida de novo; o caminho de uma sessão não muda (o registro só reusa quando já há outra).
Critério de aceite (≥ 25%): atendido. A 1ª sessão não mudou (+0,5%, ruído).

Screenshot: as duas sessões saem idênticas (`cmp`) e idênticas à rodada base (39 082 bytes).

## Limites

- A meta da ADR 030 (6 sessões em 200 MB, ≈ 33 MB marginal) não é atingida: o custo
  marginal ficou em ≈ 80 MB. O restante está em estado por sessão (heap de objetos, cache de
  código decodificado, resolve_cache, buffers de tela). Esse é o próximo passo (ADR 030, opção
  2 e layout de campos por classe).
- Medição em debug. Release não roda nesta sandbox (ver ADR 030).
