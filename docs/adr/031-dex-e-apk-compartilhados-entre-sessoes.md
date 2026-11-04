# ADR 031 — DEX e APK compartilhados entre sessões do `rd mcp`

- **Status:** proposto (desenho e critérios de aceite; implementação em ADR seguinte ou
  no mesmo PR, após revisão) · **Data:** 2026-11-04
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
