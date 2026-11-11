# ADR 037 — Stubs Android dirigidos por tabela, sem relocação e com erro delgado

- **Status:** implementado (PRs da Issue #227) · **Data:** 2026-11-10
- **Escopo:** `crates/rd-vm/src/android_stub_tables.rs` (novo),
  `crates/rd-vm/src/android_stub_surfaces.rs`, `crates/rd-vm/src/android_sysservices.rs`,
  `crates/rd-vm/src/lib.rs`
- **Relacionados:** ADR 027 (superfícies de stub), ADR 028 (nomes internados),
  ADR 034 (caminho quente de invoke), Issue #227

## Contexto

`android_stub_surfaces.rs` tinha 55 275 linhas e ~2,54 MB de símbolos `.text` no binário
`rd` de release. O grosso do arquivo eram routers escritos à mão: centenas de funções
`partial*`/`p3x_*`/`p4x_*`/`p5x_*` que abriam com `match (name, sig)`, comparavam string
por string e terminavam em corpos de 1–4 linhas (`Ok(Value::Null)`, `put` de um campo,
`Value::Int(k)`).

Cada uma dessas comparações compilava para checagem de comprimento seguida de comparação
SSE (`pcmpeqb` com máscaras em `.rodata.cst16`) ou `memcmp`, e cada corpo virava um bloco
de código próprio. O resultado era um binário de release de **24 329 936 bytes** com uma
massa de `.text` que crescia a cada superfície nova.

O requisito da Issue #227 é reduzir **1,0 MB a 1,5 MB** sem tocar em `Cargo.toml`, sem
regressão em `cold26` e mantendo as máquinas com estado (`ValueAnimator`/`ObjectAnimator`,
`android.transition`, `android.system.Os`/VFS, `WebView` history) imperativas.

## Decisão

### 1. Descritores empacotados em tabelas estáticas sem relocação

As descrições declarativas (constantes estáticas, conjuntos de classes aceitas, rotas de
classe, getters, setters, steps de `Builder`, `equals`, `hashCode`, `toString`) passaram a
viver em arrays de tipos `Copy` em `android_stub_tables.rs`:

```text
PackedStaticClass { owner: StrId, start: u16, len: u16 }
PackedStaticField { field: StrId, kind: u8, payload: i32 }
PackedMethod      { name: StrId, sig: StrId, op_kind: u8, op_aux: u16, op_arg: i32 }
```

Nenhum tipo de tabela contém `&'static str` ou `&'static [T]`. Em PIE x86-64 um fat
pointer estático gera **uma relocação `R_X86_64_RELATIVE` de 24 bytes em `.rela.dyn` mais
8 bytes de slot em `.data.rel.ro`**. A primeira tentativa (Pass 1) usou fat pointers e
gastou `+301 928` bytes só em relocação, anulando o ganho de `.text`.

### 2. `STR_POOL` plano + `StrId(u32)`

Toda string referenciada por tabela vive num único `static STR_POOL: [u8; N]` e é
referenciada por `StrId = (offset << 13) | len`. O array de bytes não precisa de
relocação; o índice é `u32` e o comprimento é limitado a 8 191 bytes (nenhum nome de
classe ou assinatura chega perto).

### 3. `StubExit(Box<VmExit>)` como canal de erro interno

`VmExit` tem 96 bytes. Como `Value` carrega `StrPlaceholder(Box<String>)`
(`BackendRepr::Memory`), todo `Result<Value, VmExit>` é retornado por `sret` e cada `?`
gerava cópia de 96 bytes mais `movups` de registradores XMM — inclusive no caminho feliz.
Internamente ao módulo os stubs passaram a usar `StubResult<T> = Result<T, StubExit>`,
onde `StubExit` embrulha um `Box<VmExit>`. O caminho `Ok` volta a ser um ponteiro
delgado em `%rax`; a caixa só existe quando há exceção. As conversões
`From<{RdError, Throwable, String, OomError, VmExit}>` continuam válidas para `?` porque
`StubExit` tem impls dedicados — `From<T> for Box<T>` **não** compõe e não deve ser
usado aqui.

### 4. Outlining dos auxiliares repetidos

- `varg(args, i)` (`#[inline(never)]`) substitui `args.get(i).cloned().unwrap_or(Value::Null)`
  e os quatro apelidos `p36a_value`/`af_arg_value`/`p35_arg`/`p41_arg` — 794 sítios de
  chamada passam a compartilhar uma cópia, em vez de cada um expandir `Option::cloned`
  mais o caminho de alocação e unwind de `Value::clone`.
- 33 auxiliares minúsculos marcados `#[cold]` (`field_int` ×757, `bool_value` ×284,
  `jstr` ×269, `field_obj` ×234, `class_of` ×141, `req_obj` ×110, …) receberam
  `#[inline(never)]`. `#[cold]` sozinho no `1.99.1` estável não implica `-Oz`, e o
  inliner duplicava esses corpos em centenas de chamadores.
- 120 formatadores de erro `#[cold] #[inline(never)] *_fmt(format_args!(...))` concentram
  o custo de `format!` (e o `.gcc_except_table` correspondente) fora dos routers.

### 5. O que permanece imperativo

`ValueAnimator`/`ObjectAnimator`, `android.transition`, `android.system.Os` com a VFS em
memória, `WebView` history, `Keystore` e os `Parcel` stateful continuam escritos à mão.
São máquinas de estado com ordem de efeito observável; nenhuma tabela as substitui.

### 6. Dobra de código idêntico no link (`--icf=all`)

O último passo não é uma transformação em Rust: é o *Identical Code Folding* do
`rust-lld`, habilitado em `.cargo/config.toml` **escopo `x86_64-unknown-linux-gnu`**
(requer `rust-lld`, que é o linker padrão desse alvo desde o Rust 1.90; em alvos cujo
linker não entende a flag o build continua funcionando).

Depois de tudo acima, ainda existe código de máquina duplicado que nenhuma refatoração
em Rust remove: corpos de função **distintos no fonte** que o LLVM compila para bytes
idênticos — stubs de erro frios, adaptadores de `Result`, `drop_glue` de tipos
monomorfizados e (o grosso) fusão em cascata, em que duas funções passam a ser idênticas
porque os seus chamados já foram fundidos. Medido em `--all-features` sobre `c4099e0`:

| Seção | sem ICF | com ICF | Δ |
|---|---|---|---|
| `.text` | 17 740 701 | 17 474 877 | −265 824 |
| `.eh_frame` | 1 364 292 | 1 288 972 | −75 320 |
| `.eh_frame_hdr` | 137 828 | 125 364 | −12 464 |
| `.relro_padding` | 2 744 | 88 | −2 656 |
| `.rela.dyn` | 572 472 | 572 424 | −48 |
| **arquivo** | **23 586 896** | **23 233 200** | **−353 696** |

Segurança: `--icf=all` funde dois corpos idênticos, o que só é observável se o código
comparar **identidade de ponteiro de função**. O repositório não tem nenhuma ocorrência
disso (`fn ... as usize`, `ptr::eq` sobre `fn`, `std::ptr::fn_addr_eq`) em `crates/`, e a
suíte `gc_host_and_jni` (7 testes, incluindo `RegisterNatives` e ponteiros JNI) passa com
o folding ativo. `cold26` não regride: mediana 12,28 ms sem ICF contra 12,33 ms com ICF em
3×7 execuções, ambas muito abaixo do orçamento de 26 ms.

## Consequências

Medições de `target/release/rd --all-features` (`lto = "thin"`), com `109/109` testes de
`android_stub_surfaces` e `228` testes de lib verdes em todas as etapas:

| Passo | Tamanho (B) | Δ vs `main` |
|---|---|---|
| `main` @ `d1ee058` | 24 329 936 | — |
| Pass 1 — extração ingênua (fat pointers) | 24 207 552 | −122 384 |
| Pass 2 — `STR_POOL` sem relocação | 23 841 168 | −488 768 |
| Pass 3 — `toString`/`equals`/`hashCode`/steps declarativos | 23 880 544 | −449 392 |
| Pass 4 — `StubExit(Box<VmExit>)` + formatadores outlined | 23 611 344 | −718 592 |
| Pass 5 — `varg` + `#[inline(never)]` nos auxiliares | 23 586 896 | −743 040 |
| Pass 6 — `--icf=all` no `rust-lld` | **23 233 200** | **−1 096 736 (−1,046 MB)** |

O arquivo-fonte caiu de 55 275 para ~51 300 linhas. O código restante é mais fácil de
ler: um router é uma tabela mais um laço, não 300 braços de `match`.

## Onde os bytes restantes estão (e por que parar aqui)

`android_stub_surfaces` + `android_stub_tables` somam 1 843 169 B. A análise do mapa de
linker e da fonte mostra que **o poço da conversão declarativa secou**:

- **1 481 braços** restantes dentro de `match (name, sig)`, com **1 265 corpos distintos**
  depois de normalizar literais e números; só 97 braços compartilham corpo com outro.
- Em 1 285 funções, apenas **25** têm corpo idêntico após normalização e **13** pares são
  quase-duplicados (Jaccard > 0,5). O que sobrou é comportamento genuinamente distinto.
- Comparação de strings responde por **2,3%** do `.text` do módulo (3 123 `pcmpeqb` em
  354 912 instruções), porque o LLVM já troca a maioria dos `match` por `cmp` imediato e
  tabelas de salto. Trocar dispatch por hash `u64` renderia ~30–40 KB, não centenas.

Por isso o passo final é de linker: o que sobra é duplicação que se forma **depois** da
geração de código, não antes.

## Critérios de aceite

1. `cargo test -p rd-vm --test android_stub_surfaces` verde (109/109).
1b. Redução medida ≥ 1,0 MB no binário `target/release/rd --all-features`: **−1 096 736 B
   (−1,046 MB)**.
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings` limpo.
3. Zero `unsafe` fora de `rd-jni`/`rd-ndk`.
4. `cold26` sem regressão (binário menor e com menos relocações só pode ajudar o
   carregamento, mas o snapshot mediano continua sendo o gate).
5. Testes unitários em `android_stub_tables` cobrindo o empacotamento de `StrId`, a
   ordenação dos ranges de busca binária e a decodificação de todas as strings de tabela.

## Riscos e limites

- O `StrId` embute o comprimento em 13 bits. Uma string de tabela acima de 8 191 bytes
  truncaria silenciosamente; o teste `str_id_packing_round_trips` e
  `every_table_string_decodes_inside_the_pool` existem para pegar isso.
- Os ranges de busca binária são gerados; se um deles deixar de estar ordenado, membros
  passam a ser reportados como estranhos. `class_set_ranges_are_sorted_and_answer_membership`
  e `class_router_ids_are_dense_and_reachable` cobrem os ranges exercitados.
- Tabelas com chave `(name, sig)` não aceitam braços com curinga parcial
  (`("<init>", _)`); esses continuam sendo `match` à mão.
