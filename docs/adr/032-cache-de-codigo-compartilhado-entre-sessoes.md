# ADR 032 — Cache de código decodificado compartilhado entre sessões

- **Status:** proposto (desenho e critérios de aceite; implementação em PR de código
  separado, após esta ADR ser mergeada) · **Data:** 2026-11-04
- **Escopo:** `crates/rd-vm/src/classpath.rs` (`code_cache`, `class_data_cache`, `code()`,
  `class_data()`), `crates/rd-agent/src/apk_share.rs` (`SharedApk`)
- **Relacionados:** ADR 030 (opção 2: código decodificado), ADR 031 (DEX compartilhados,
  Weak), ADR 029 (composição do RSS), Lei 2 (bytes brutos, incluindo `CodeItem::insns`)

## Contexto

Depois da ADR 031, o custo marginal de uma sessão extra é ≈ 81 MB (medido: 2 sessões
257 464 KB, 1 sessão 176 552 KB). O que sobra por sessão é estado de execução. O item
"código decodificado" foi estimado na ADR 030 em até ~10 MB por sessão.

Fatos do código atual (verificados em `classpath.rs`):

- `code_cache: HashMap<(usize, usize), Arc<CodeItem>>`, chave `(dex_idx, code_off)`. É
  por `Classpath`, então cada sessão decodifica de novo os mesmos métodos do mesmo APK.
- `class_data_cache: HashMap<(usize, u32), Option<Arc<ClassData>>>`, mesma situação.
- `CodeItem` e `ClassData` são imutáveis depois de criados. Quem os cria é uma função pura
  dos bytes do DEX. Um DEX compartilhado (ADR 031) dá o mesmo resultado para todas as
  sessões.

Medição de um processo (perfil de heap de `fd-dbg3`, bytes pedidos, não residentes):
19 653 corpos decodificados, 261 327 instruções, `Vec<(usize, Insn)>` de 10,6 MB. É um
processo só, não duas sessões. Por isso a economia abaixo é estimativa.

## Opção escolhida

1. O cache de código e o de `ClassData` saem do `Classpath` e vão para um objeto
   compartilhado por conjunto de DEX (`SharedCode`), guardado ao lado de `SharedApk`.
   A chave passa a ser `(dex_idx, offset)` dentro desse conjunto, então o `dex_idx` continua
   válido porque o conjunto é o mesmo.
2. O objeto é mantido por `Weak`, como os DEX na ADR 031. Quando o último engine para, a
   memória volta.
3. `Classpath::code()` e `class_data()` mudam pouco: o `Classpath` guarda um
   `Arc<SharedCode>` (ou um cache local quando não há compartilhamento) e a API pública
   não muda.
4. Escrita sob `Mutex`. Duas sessões que decodificam o mesmo método podem competir; o
   resultado é idêntico (função pura), então a segunda escrita descarta a sua.

## Riscos

- **Determinismo:** o resultado de decodificar depende só dos bytes. Os testes de
  golden e de integração já cobrem isso, mas a medição de dois engines lendo o mesmo
  método deve ser comparada byte a byte.
- **Contenção:** um `Mutex` por DEX. Sessões na mesma thread não competem; o `rd mcp`
  atende em uma thread.
- **Retenção:** enquanto um engine roda, o cache fica vivo. Não aumenta o pico de uma
  sessão só; aumenta o pico só se o cache crescer mais que um engine usaria.
- **Lei 2:** nada é descartado. `CodeItem::insns` continua inteiro; só muda quem é dono.

## Critérios de aceite

- Custo marginal da 2ª sessão cai ≥ 10 MB em relação aos 80,9 MB medidos (meta:
  ≤ 71 MB), mesmo driver `drive.py`, mesmo APK, pareado com a base.
- Screenshots de 1 e 2 sessões idênticas (`cmp`).
- Testes: `rd-vm` (lib + integração + golden), `rd-agent` (lib + integração). Teste novo:
  dois `Classpath` sobre o mesmo `SharedCode` devolvem o mesmo `CodeItem` (`Arc::ptr_eq`)
  e o mesmo `ClassData`.
- Sem mudança de API pública de `Classpath`.

## Não é objetivo desta ADR

- Layout de campos por classe (ADR 030, opção 3). Vai numa ADR própria depois, com a
  caracterização de duas sessões.
- Medição em release (bloqueada nesta sandbox; ver ADR 030).

## Consequências

- Se a meta for atingida, o custo marginal fica perto de 70 MB. Para 6 sessões em 200 MB
  seriam necessários ≈ 33 MB por sessão. A distância restante exige atacar o estado de
  objetos (ADR 030, opções 3 e 4).
