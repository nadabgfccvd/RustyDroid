# ADR 029 — Composição do RSS e próximos passos de memória

- **Status:** proposto (medição e plano; nenhum código de representação muda nesta ADR)
  · **Data:** 2026-11-03
- **Escopo:** diagnóstico de memória do primeiro quadro do F-Droid; alvo de sessões
  simultâneas por teto de RAM
- **Relacionados:** ADR 008 (DEX lazy), ADR 015 (teto de RAM), ADR 023 (orçamento de
  heap por sessão), ADR 024 (sessões isoladas), ADR 028 (valores 16 B, nomes internados)

## Contexto

O ADR 028 reduziu o pico de RSS do primeiro quadro do F-Droid de 196 MB para 164 MB
(−16,5%, build debug). O alvo de trabalho ("2 → 6 sessões simultâneas por 200 MB") pede
cerca de 3× menos memória por sessão. Antes de escolher o próximo passo, foi medida a
composição da memória viva em malloc no mesmo quadro.

## Medição

Ferramenta: amostrador de malloc com `LD_PRELOAD`, fora do repositório
(`/home/user/logs/heapprof/`). Alocações de 32 KB ou mais são contadas exatas; as menores
são amostradas em 1 de 16 e ponderadas. Binário: `target/debug/rd` do commit `2f70bc4`.
Saída no instante em que o PNG é escrito (antes do desmonte da VM, que libera tudo).

Ressalva: o amostrador mede bytes **pedidos**, não páginas residentes. Folga de capacidade
de `Vec` em páginas não tocadas não entra no RSS. Os valores abaixo são limites superiores.

Primeira medição (só amostragem 1/16, sem exatidão nas grandes alocações) atribuiu 8 MB
ao índice de classes. **Essa atribuição era ruído de amostragem e foi retirada.** Na
medição exata, o índice de classes não aparece entre as maiores origens.

| Origem (sítio de alocação) | Bytes vivos pedidos | Observação |
|---|---|---|
| Bytes descomprimidos dos 3 DEX (`Zip::read_entry_whole`) | 20,1 MB | 9,3 + 9,6 + 1,7 MB; é o maior bloco fixo |
| APK inteiro lido em memória (`Apk::open` via `std::fs::read`) | 12,3 MB | mantido durante a sessão; ver "Ação 1" |
| Instruções decodificadas `Vec<(usize, Insn)>` | 10,6 MB | conteúdo real: 261 327 × 32 B = 8,4 MB; o resto é folga de crescimento |
| Campos de instância `Vec<(Arc<str>, Value)>` | 8,6 + 4,9 MB | layout por objeto; ADR futuro |
| `Vec<HeapObj>` (tabela de objetos) | 7,2 MB | 56 B por objeto, com folga de crescimento |
| `Vec<Value>` por `repeat_n` (arrays ou tabelas de valores) | 6,4 MB | dono ainda não identificado |
| Cache de resolução `(String, String, String)` (`resolve_cache`) | 4,9 MB | chaves de `String` triplas; candidato barato |
| Regex Unicode estáticas (`numerics.rs`) | ~3,3 MB | `\p{L}`, `\p{Cf}`, `\p{Mn}` |

Total amostrado: ~123 MB (com a folga de capacidade e o APK inteiro). O heap contabilizado
da VM (`[rd mem]`) é 19,1 MB em 109 639 objetos; 19 653 corpos decodificados, 261 327
instruções.

## Opções avaliadas

1. **APK inteiro na memória.** `Apk::open` lê o arquivo todo (12,3 MB) e o mantém, além
   dos DEX descomprimidos. O `rd-apk` já tem leitor paginado (`open_paged`, com a mesma
   política de limites e CRC). Ação: abrir o APK do caminho de execução com o leitor
   paginado e validar com os testes de paridade existentes. Ganho estimado: até ~12 MB
   por sessão. É o passo mais barato da lista.
0. **Cache de resolução.** `resolve_cache` guarda ~4,9 MB com chaves de três `String`.
   Trocar a chave por `(dex, method_idx)` ou por `Arc<str>` reduz esse custo.
2. **Código decodificado.** Hoje cada corpo guarda `Vec<u16>` bruto (cópia) e
   `Vec<(usize, Insn)>` com 32 B por instrução. Opções: descartar o bruto no caminho da
   VM (≈1 MB, sem efeito nos testes de round-trip do `rd-dex`, que usam o `CodeItem`
   direto); endereço `u32` e `Insn` de 16 B (exige boxing de variantes de 3 operandos,
   mudança grande no decodificador e no interpretador); decodificação sob demanda com
   descarte de corpos frios (maior ganho, maior risco de desempenho).
3. **Buffers de DEX.** ~14 MB de bytes descomprimidos são necessários para ler o DEX.
   Reduzir exige mapeamento de arquivo (`mmap`), o que hoje é proibido pela política
   `unsafe_code = "forbid"`, ou ler entradas `STORED` por deslocamento. Sem mudança de
   política, não há ganho aqui.
4. **Objetos do heap.** 109 639 objetos e 19 MB contabilizados. Layout de campos por
   classe (slots em vez de `Vec<(Arc<str>, Value)>` por instância) reduziria a sobrecarga
   por objeto. É refatoração ampla da VM.
5. **Regex estáticas.** ~3 MB. Trocar por tabelas próprias muda a semântica Unicode;
   não é feito sem testes de paridade específicos.
6. **Compartilhamento entre sessões.** Cada sessão hoje monta seu próprio `Classpath`.
   Se o `DexBacking::Owned(Arc<[u8]>)` for compartilhado entre sessões do mesmo APK,
   os ~14–20 MB de DEX e o índice de classes deixam de ser cobrados por sessão. Este é,
   provavelmente, o maior ganho para o alvo de sessões simultâneas. **Não medido**: exige
   um teste com duas sessões no mesmo processo.

## Decisão

Nesta rodada, não se altera representação. Próximos passos, em ordem:

1. ~~Abrir o APK com leitor paginado~~: medido, sem ganho (172 MB contra 169 MB); revertido.
0. Reduzir a chave do `resolve_cache` (~4,9 MB).

**Resultado da ação 0 (medido):** chave do `resolve_cache` com `Arc<str>` de um
interner por `Classpath` (PR #525). Pico com `rd app shot` (debug, `MALLOC_ARENA_MAX=1`,
dois pares de execuções alternadas): base 169 216 e 165 012 KB (média 167 114); mudança
163 156 e 162 680 KB (média 162 918). Redução média de 4,2 MB (−2,5%), menor que a
estimativa. Captura idêntica.

**Resultado da ação 1 (medido, sem ganho):** `Apk::open_paged` no caminho da CLI deu
pico de 172 168 KB contra 169 252 KB da v5 (mesmo modo, `MALLOC_ARENA_MAX=1`, debug).
Saída idêntica (39 082 bytes). A mudança foi revertida. A estimativa de 12 MB foi
refutada pela medição.
2. Medir duas sessões no mesmo processo com o mesmo APK, com e sem `DexBacking`
   compartilhado. Esse número decide se o alvo de sessões é alcançável por
   compartilhamento.

**Resultado da ação 2 (medido, ADR 030):** `rd mcp` com 1 sessão: pico 175 744 KB; com 2
sessões: 303 748 KB. Custo marginal da 2ª sessão ≈ 128 MB. O alvo de 6 sessões em 200 MB
não é atingível só com compartilhamento; ver ADR 030.
3. ~~Descartar `CodeItem::insns` no caminho da VM~~: não feito. O código bruto é a
   cópia exigida pela Lei 2 do repositório ("nada se perde"); o ganho estimado (≈1 MB)
   está abaixo do ruído de medição (±2,5%).
4. Só depois avaliar layout de objetos e decodificação sob demanda, em ADR próprio.

## Consequências

- O alvo "6 sessões por 200 MB" não é atingido por mudanças locais de representação.
  Com ~164 MB por sessão (debug), o ganho local esperado é de poucos pontos percentuais
  por passo.
- A medição com amostrador é reproduzível fora do repositório; o relatório deve indicar
  que bytes são pedidos e não residentes.
- Números de build debug não valem como número de release. Ver `docs/gc.md`.
