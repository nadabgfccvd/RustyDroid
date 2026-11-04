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

Ferramenta: amostrador de malloc com `LD_PRELOAD` (1 em 16 alocações, pilha de 6
quadros, saída no momento em que o PNG é escrito, antes do desmonte da VM). Fica fora
do repositório, em `/home/user/logs/heapprof/`. Binário: `target/debug/rd`, mesmo
commit do ADR 028.

Ressalva importante: o amostrador mede bytes **pedidos** (`size` da alocação), não
páginas residentes. Folga de capacidade de `Vec` em páginas nunca tocadas não conta
para o RSS. Por isso os números abaixo são limites superiores do que é residente.

| Origem (sítio de alocação) | Bytes vivos pedidos (aprox.) | Observação |
|---|---|---|
| Buffers de inflate do DEX (`miniz_oxide` via `read_entry`) | 14 MB | bytes realmente escritos; é a maior parte fixa |
| Instruções decodificadas `Vec<(usize, Insn)>` | 10,9 MB | conteúdo real: 261 327 × 32 B = 8,4 MB |
| `lazy::ClassSlot` (índice de classes, `cap = 2n` arredondado) | 8,2 MB | 16 alocações de 512 KB; ver "Ação 1" |
| `Vec<Value>::reserve` | 6,5 MB | dono ainda não identificado |
| Índices eager de field/proto ids (`Vec<FieldId>`, `Vec<ProtoId>`) | 3,8 + 3,4 MB | tabelas `rd-dex` em `Dex::parse` |
| `Arc<CodeItem>` / `Arc<MethodRef>` / `StringCachePage` | 2,4 MB cada | caches de execução |
| Regex Unicode estáticas (`numerics.rs`, `OnceLock`) | ~3,3 MB | `\p{L}`, `\p{Cf}`, `\p{Mn}` |

Total amostrado: ~75 MB, próximo do `[heap]` de 77 MB medido por `smaps` no ADR 028.
O heap contabilizado da VM (`[rd mem]`) é 19,1 MB em 109 639 objetos; decodificação de
código: 19 653 corpos e 261 327 instruções.

## Opções avaliadas

1. **Índice de classes (`ClassSlot`).** 8 MB de bytes pedidos para ~16 alocações
   sugerem mais de um `Dex` com índice construído, ou classes demais por DEX. Ação:
   contar construções de `RawClassIndex` por execução e verificar se há `Dex`
   duplicados no `Classpath`. Se houver duplicação, é o ganho mais barato.
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

1. Contar construções de `RawClassIndex` e `Dex` por execução do F-Droid; corrigir se
   houver duplicação (medir antes e depois).
2. Medir duas sessões no mesmo processo com o mesmo APK, com e sem `DexBacking`
   compartilhado. Esse número decide se o alvo de sessões é alcançável por
   compartilhamento.
3. Descartar `CodeItem::insns` no caminho da VM (≈1 MB), se o golden continuar verde.
4. Só depois avaliar layout de objetos e decodificação sob demanda, em ADR próprio.

## Consequências

- O alvo "6 sessões por 200 MB" não é atingido por mudanças locais de representação.
  Com ~164 MB por sessão (debug), o ganho local esperado é de poucos pontos percentuais
  por passo.
- A medição com amostrador é reproduzível fora do repositório; o relatório deve indicar
  que bytes são pedidos e não residentes.
- Números de build debug não valem como número de release. Ver `docs/gc.md`.
