# ADR 028 — Memória compacta: valores de 16 bytes, nomes internados, código decodificado menor

- **Status:** aceito (implementado no branch `perf/memoria-compacta-s11`; ainda não
  mergeado — ver a seção "Validação") · **Data:** 2026-11-03
- **Escopo:** `crates/rd-vm/src/value.rs` (`Value::StrPlaceholder`),
  `crates/rd-vm/src/heap.rs` (`HeapObj::Instance`, `Heap::intern`, `free_slot`),
  `crates/rd-vm/src/classpath.rs` (`decoded_code_stats`),
  `crates/rd-dex/src/code.rs` (`Kind::Payload` boxed, `Kind::payload()`),
  `crates/rd-cli/src/main.rs` (`dexes_of_apk` com `parse_unverified`; `RD_MEM_REPORT`)
- **Relacionados:** ADR 006 (GC geracional), ADR 008 (parse lazy de DEX, estágio 1b),
  ADR 023 (orçamento de heap por sessão), ADR 015 (teto de RAM do processo)
- **Ordem:** decisão registrada antes do código de representação (CONTRIBUTING).

## Contexto

O objetivo do dono foi "mais unidades de trabalho com a mesma RAM": com 200 MB, o
mesmo runtime deveria sustentar 6 unidades (sessões de app) no lugar de 2. Antes de
mudar qualquer coisa, medimos o primeiro quadro do F-Droid real (`rd app shot`,
build debug, pico de RSS do processo, `VmHWM`):

| medida | valor |
|---|---|
| RSS de pico, linha de base (`5751d6b`) | 196 096 KB |
| heap da VM contabilizado no fim do quadro | 19,1 MB (109 639 objetos, 8 406 nomes) |
| mapa de memória (amostra, 200 s) | `[heap]` do malloc 77 MB; regiões anônimas 25 MB e 14 MB (DEX brutos); texto do binário debug 19 MB |
| DEX do APK, descompactados | 20,6 MB (9,3 + 9,6 + 1,7) |
| corpos de método decodificados no quadro | 19 653 corpos, 261 327 instruções |
| tamanho de `Value` / `Insn` / `(usize, Insn)` | 24 / 56 / 64 bytes |

Achados:

1. O heap contabilizado (19 MB) não explica o RSS (196 MB). A maior parte da memória
   está fora do heap do app: DEX brutos, tabelas derivadas e código decodificado.
2. Cada elemento de array e cada registrador ocupa um `Value`, que tinha 24 bytes por
   causa da variante `StrPlaceholder(String)`.
3. Cada instância guardava uma cópia própria do nome da classe e de cada campo
   (`String`), e cada escrita de campo novo alocava outra cópia.
4. Cada instrução decodificada ocupava 64 bytes para 2–6 bytes de código bruto, porque
   `Payload` (dois `Vec`) era embutido em `Kind`.
5. O coletor, nesse quadro, rodou só 2 ciclos (cerca de 35 ms cada). O custo de
   memória não está no coletor; está na representação dos dados.
6. A CLI rodava o caminho eager de DEX (todas as strings decodificadas), enquanto o
   agente MCP já usava `parse_unverified` (ADR 008, estágio 1b).

## Decisão

1. **`Value::StrPlaceholder(Box<String>)`.** `Value` cai de 24 para 16 bytes. Arrays,
   registradores e campos ficam 33% menores. O texto é a exceção (caminho de
   `parse_arg_value`), então o custo extra do `Box` não pesa.
2. **Nomes internados.** `Heap` mantém um conjunto de `Arc<str>`. Classes de instância
   e nomes de campos são `Arc<str>` compartilhados; `alloc_instance` e `put_field`
   internam. A assinatura pública de `alloc_instance(String, Vec<(String, Value)>)`
   não muda, para não tocar nos ~680 pontos de chamada.
3. **Tombstone sem alocação.** `free_slot` reutiliza o nome interno de
   `FREED_CLASS`, sem alocar um `Arc` novo a cada varredura.
4. **`Kind::Payload(Box<Payload>)`** com o acessor `Kind::payload()`. `Insn` cai de 56
   para cerca de 24 bytes, e a entrada decodificada de 64 para cerca de 32 bytes.
5. **CLI com `parse_unverified` no caminho de execução** (`dexes_of_apk`). O checksum
   do header e a decodificação das strings ficam para o acesso, como no MCP. O
   `rd dex` / `inspect` continuam usando `parse`, que é quem reporta `checksum_ok`.
6. **Diagnóstico `RD_MEM_REPORT=1`** no `rd app shot`: imprime o heap (usado, pico,
   objetos, slots livres, nomes internados) e os corpos decodificados. Fica desligado
   por padrão.

**Não decidido aqui:** arrays tipados (`Vec<i32>`, `Vec<u8>`) em vez de `Vec<Value>`
— seria a maior economia para arrays de bytes, mas toca toda a API de `array_elems_mut`.
Também não muda o orçamento contabilizado (`cost` ainda estima 4/8 bytes por elemento);
fica como trabalho futuro, porque o orçamento hoje é um teto, não uma medida de RSS.

**Configuração de alocador (sem mudança de código):** `MALLOC_ARENA_MAX=1` reduz o RSS em
cerca de 4% no primeiro quadro, porque a VM roda numa thread separada e o glibc cria uma
arena extra com blocos de 64 MiB. O workspace proíbe `unsafe`, então isso fica como
variável de ambiente documentada, não como `mallopt` no código.

## Medidas (debug, `rd app shot` do F-Droid, pico de RSS do processo)

| versão | RSS (KB) | variação |
|---|---|---|
| linha de base (`5751d6b`) | 196 096 | — |
| + `Value` 16 bytes, nomes internados | 186 896 | −4,7% |
| + `MALLOC_ARENA_MAX=1` | 178 812 | −8,8% |
| + `parse_unverified` no caminho de execução | 180 648 (com arena 1) | ruído |
| + `Kind::Payload` boxed (`Insn` menor) | 163 808 | −16,5% |
| medida de controle com o contador de código | 169 252 | ruído de ±3% |

As imagens de saída são idênticas em todas as versões (`39 082` bytes, mesmos pixels).
Os números são de build debug; o perfil `release` (thin LTO) não foi medido.

## Consequências

- Ganho medido de cerca de 16% de RSS no primeiro quadro. **Não** chega aos 3x do
  objetivo. Para chegar lá, o próximo passo mais provável é: arrays tipados e
  decodificação de código sob demanda com descarte (o código decodificado ainda é a
  maior parte do malloc).
- Compatibilidade: o comportamento observável não mudou; as imagens são idênticas. A
  mudança de `parse` para `parse_unverified` remove a checagem antecipada de strings
  MUTF-8 no caminho de execução. Um DEX com string inválida agora falha no primeiro
  acesso, não no carregamento. É o mesmo contrato que o agente MCP já tinha.

## Validação

Rodada em `2026-11-03`, no branch de trabalho (build debug, `-j 1`):

- `cargo fmt --all -- --check`: limpo.
- `cargo clippy --workspace --all-targets -- -D warnings`: limpo (inclui os alvos de teste).
- `cargo test -p rd-dex --lib`: 104 passed, 0 failed.
- `cargo test -p rd-vm` (lib e todos os testes de integração): 944 passed, 0 failed.
- `cargo test -p rd-cli --test fdroid_oncreate_e2e`: 2 passed (`onCreate` do F-Droid e primeiro quadro).
- Golden suite VM com JDK 21 (`golden/vm/run-golden.sh`, debug): 150/150 PASS.
- Primeiro quadro do F-Droid: a imagem de saída é idêntica à da linha de base (39 082 bytes, mesmos pixels).

Não rodado nesta rodada: workspace completo (`cargo test --workspace`) e o perfil
`release`. Ficam como pendências no PR.
