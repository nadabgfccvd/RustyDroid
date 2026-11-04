# ADR 035 — Identidades inteiras (classe, campo, método) no caminho quente

- **Status:** proposto (desenho, evidências e critérios de aceite; implementação em PRs
  de código separados, depois desta ADR ser mergeada) · **Data:** 2026-11-04
- **Escopo:** `crates/rd-vm/src/classpath.rs`, `heap.rs` (`HeapObj::Instance`), `interp.rs`
  (invoke, iget/iput), `android_stub_surfaces.rs` e demais gates de host que comparam nomes
- **Relacionados:** ADR 034 (caminho de invoke: cache por site, buffers de frame), ADR 033
  (layout de campos por classe: a mesma peça estrutural, vista por outro lado), ADR 028
  (nomes internados)

## Contexto

A ADR 034 reduziu o custo de alocação e de resolução repetida, com ganhos de 5–9%. Os
alvos do usuário (`launch` 450 ms, ferramenta 345 ms, medidos com observe on) continuam
~1,33× a ~1,4× abaixo do atual.

O perfil de `rd-s12` (DWARF, `tap_text` observe on) mostra o que resta. Os itens abaixo são
**comparações de nome ou buscas por string** (amostras do perfil, não somadas com dupla
contagem):

| Item | Amostras (≈) |
|---|---|
| `Classpath::is_subtype` (memo por `&str`) | 4,6% |
| `classpath::intern_name` | 1,8% |
| `classpath::field_ref_shared` (RefCell + `Arc` por acesso) | 1,8% |
| `Heap::class_arc_of` (`Arc` clone por instância) | 1,6% |
| `classpath::method_ref` | 1,5% |
| `classpath::find_class` | 1,0% |
| `Heap::get_field` (varredura por nome) | 0,9% |
| `memcmp` da libc (chamado por `==` de `str`) | ~5% |
| `hash_one` (SipHash/Fx em chaves de string) | ~0,9% |

Somados, cerca de 20% das amostras vêm de lookups por nome. Já o trabalho de interpretação
(`exec_frame` + `exec_leaf_insn`) é cerca de 16% de auto-tempo. Isso bate com a observação
da ADR 034: o custo é o app executando muitas chamadas pequenas.

Restrições medidas:

- `==` em `str` não tem atalho por ponteiro: compara o conteúdo sempre que os tamanhos
  batem, e isso chama `memcmp`. Por isso um cache que testa identidade e depois conteúdo,
  para cada entrada, paga `memcmp` em quase toda consulta. Um identificador inteiro evita
  isso por completo.
- `is_subtype` tem chamadores concentrados em gates de host (`is_thread_class`,
  `is_context`, `instance_intrinsic`), cada um com poucos por cento.

## Opção proposta

Trocar nomes por identificadores inteiros **no caminho quente**, mantendo nomes como fonte
de verdade para diagnóstico e para o JSON das ferramentas MCP.

1. **`ClassId(u32)`**: `Classpath` mantém a tabela nome ↔ id (criada na montagem, com os
   nomes de DEX e de plataforma já conhecidos). `HeapObj::Instance { class }` passa a
   guardar `ClassId`; `class_arc_of` vira cópia de inteiro.
2. **Ancestrais por classe**: `is_subtype(ClassId, ClassId)` consulta um vetor de bits de
   ancestrais, calculado uma vez por classe (substitui o memo de strings).
3. **Campos e métodos resolvidos por site**: iget/iput e invoke guardam `(ClassId, FieldSlot)`
   e `(ClassId, MethodId)` resolvidos; a memo da ADR 034 deixa de comparar strings.
4. **Gates de host** comparam `ClassId` com constantes resolvidas na inicialização
   (`once`), em vez de `&str` a cada chamada.
5. **Layout de campos por classe** (ADR 033) entra junto no passo 3: o slot de campo deixa
   de ser buscado por nome.

## Critérios de aceite

1. Nenhuma saída de ferramenta MCP muda (teste de paridade de JSON e `seq_compare.py`).
2. `cargo test -p rd-vm --lib` (193) e `-p rd-agent --lib` (112) passando, mais testes novos
   para: ancestrais por bitset iguais ao `is_subtype` atual (amostra de pares de classes do
   Comprix) e resolução de campo sombreado (`decl#nome`).
3. `comprix-regression.py --cycles 2` com sucesso (17 passos).
4. Benchmark pareado (mesma caixa, n ≥ 6, pares alternados). Meta do passo: ≥ 15% de
   redução em `tap_text` e `launch` com observe on, sem regressão em `key`.
5. Relatório com os alvos (450 ms e 345 ms) e a distância restante.

## Estimativa honesta

Se os ~20% de lookups por nome forem eliminados quase por completo, o ganho esperado é de
~1,25×. Isso leva `launch` a ~480 ms e a ferramenta a ~380 ms (observe on): ainda acima dos
alvos. Chegar a 450 ms e 345 ms exige, além disto, reduzir o custo de interpretação
(`exec_frame`, `exec_leaf_insn`): instruções pré-decodificadas com alvos já resolvidos e
menos estado por instrução. Essa parte fica para uma ADR própria (036), depois desta.

## Não-objetivos

- Não mudar o modelo de observe nem o formato das ferramentas MCP.
- Não reescrever o interpretador nesta ADR.
- Não remover os nomes: eles continuam para diagnóstico (`label_of`) e para o JSON.

## Riscos

- O refator toca cerca de 140 pontos que usam `class_of`/`&str` de classe. Deve ser feito em
  PRs pequenos, cada um com a paridade de ferramentas validada.
- IDs são por sessão de `Classpath`. Não devem vazar para o estado persistente do cache
  compartilhado (ADR 032) sem uma chave estável.
