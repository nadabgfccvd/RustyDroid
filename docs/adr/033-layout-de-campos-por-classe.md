# ADR 033 — Layout de campos de instância por classe

- **Status:** proposto (desenho e critérios de aceite; implementação em PR de código
  separado, depois desta ADR ser mergeada) · **Data:** 2026-11-04
- **Escopo:** `crates/rd-vm/src/heap.rs` (`HeapObj::Instance`, `alloc_instance`),
  `crates/rd-vm/src/engine.rs` e demais pontos de `iget`/`iput`
- **Relacionados:** ADR 030 (opção 3: layout por classe), ADR 029 (opção 4: nomes por
  campo), ADR 028 (valores de 16 bytes e nomes internados), ADR 032 (cache compartilhado)

## Contexto

Depois da ADR 032, o custo marginal de uma sessão extra é ≈ 61 MB (medido: 2 sessões
238 432 KB; 1 sessão 177 420 KB). O que sobra por sessão é, em parte, o estado dos objetos.

Fato do código (verificado em `heap.rs`):

- `HeapObj::Instance { class: Arc<str>, fields: Vec<(Arc<str>, Value)> }`. Cada campo guarda
  o nome internado (um ponteiro gordo de 16 bytes) junto com o valor (16 bytes).
- A contabilidade de heap usa 24 bytes por campo (`32 + fields.len() * 24`), que não bate
  com o tamanho real da entrada. Isso já é uma divergência a corrigir na implementação.
- Medição de um processo (perfil de heap de `fd-dbg3`, bytes pedidos): vetores de campos
  de instância de ~8,6 MB + ~4,9 MB; `HeapObj` de ~7,2 MB. São números de uma sessão só,
  não de duas.

## Opção escolhida

1. Cada classe ganha um **layout**: `Arc<[Arc<str>]>` com os nomes de todos os campos de
   instância (da própria classe e das superclasses), na ordem de declaração, e um índice
   nome → posição (`HashMap` ou busca linear para classes pequenas).
2. `HeapObj::Instance` passa a guardar `layout: Arc<ClassLayout>` e `values: Vec<Value>`.
   O nome sai de cada campo; fica uma vez por classe, compartilhado por todas as instâncias.
3. `iget`/`iput` resolvem o nome para posição (com cache por site de instrução quando a
   classe é a mesma). A API `Heap` mantém os métodos por nome; só a representação muda.
4. O layout é construído uma vez por classe, na primeira instanciação, a partir da
   `ClassData` (ADR 032 já guarda a `ClassData` compartilhada). Como o layout depende só dos
   bytes do DEX, ele também pode ficar no cache compartilhado.
5. Lei 2: nada é descartado da `ClassData`; só a cópia do nome por instância sai.

## Estimativa (não medida)

- Por campo: de 32 bytes (nome 16 + valor 16) para 16 bytes (valor). Economia de 16 bytes
  por campo de instância vivo.
- Ordem de grandeza para o perfil atual: metade dos ~13,5 MB de vetores de campos,
  ou seja ~6 MB por sessão com muitos objetos. Com a ADR 032, a meta de custo marginal
  ≤ 55 MB exige pelo menos isso.

## Riscos

- **Herança e sombreamento:** dois campos com o mesmo nome em classes diferentes da mesma
  hierarquia. O layout precisa de chave `(classe declarante, nome)`, não só o nome.
- **Reflexão e `Unsafe`:** qualquer caminho que lê campos por nome deve passar pelo mesmo
  resolvedor. A busca de todos os pontos (ADR 030: 393 acessos por nome) é parte da
  implementação.
- **Objetos sem classe conhecida:** instâncias criadas pelo host (`alloc_instance` com
  `String`). Precisam de um layout vazio ou de fallback.
- **Desempenho:** `iget`/`iput` quentes. O cache por site de instrução evita a busca em
  cada execução; a medição de tempo deve ser feita antes do merge.

## Critérios de aceite

- Custo marginal da 2ª sessão cai ≥ 5 MB em relação à base da ADR 032 (61 MB medidos),
  pareado, mesmo driver `drive.py`, mesmo APK, com screenshots idênticas (`cmp`).
- Testes: `rd-vm` (lib, integração, golden) e `rd-agent`. Teste novo: sombreamento de campo
  entre superclasse e subclasse lê e escreve o campo certo; instância criada pelo host
  continua funcionando.
- Nenhuma mudança de API pública de `Heap` e de `Engine` para os chamadores externos.

## Não é objetivo desta ADR

- Reduzir `Vec<HeapObj>` com folga de capacidade (ADR 030, opção 4). Vai depois, com medição.
- Medição em release (bloqueada nesta sandbox; ver ADR 030).

## Consequências

- Se a estimativa valer, o custo marginal fica perto de 55 MB. Para 6 sessões em 200 MB,
  ainda faltam ~25 MB por sessão. A conclusão da ADR 030 continua de pé: o alvo exige
  reduzir também o estado de execução que não é objeto.
