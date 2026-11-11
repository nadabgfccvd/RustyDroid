# ADR 036 — Exceção datada: fixture DEX reduzido do RxJava 3

- **Status:** proposta. Não há gate ativo: o RD Sentinel foi retirado (ADR 010) e nenhum workflow consome o label `sentinel:policy-approved`. Manter, ajustar ou descartar a exceção é decisão do mantenedor.
- **Data:** 2026-11-05
- **Escopo:** Apêndice D (S12), `androidx`/`rxjava` como `dex-passthrough`.

## Contexto

`.github/rd-sentinel/policy.json` (arquivo mantido, sem gate que o leia) lista `.dex` em `diff.binary_extensions_blocking`.
Enquanto o RD Sentinel estava ativo, um arquivo desse tipo commitado era finding bloqueante (P0/P1), e a ADR 010 exigia
`sentinel:policy-approved` aplicado por mantenedor para liberá-lo.

Testar a biblioteca real (e não um DEX sintético) é a única forma de validar que o
bytecode do RxJava executa na VM. Gerar o DEX em tempo de teste exige rede e R8 no CI, o que não é reprodutível.

## Decisão

Exceção **limitada** a um único arquivo:

| Campo | Valor |
|---|---|
| Arquivo | `crates/rd-vm/tests/fixtures/rxjava3/classes.dex` (23 KB) |
| SHA-256 | `4023c2291dd57bf6d7718da62f0b837f3fa7b45555c3ee04550c4f9f284c3fac` |
| Dono | mantenedor do repositório (`nadabgfccvd`), a confirmar na aprovação |
| Motivo | teste de passthrough com bytecode real de RxJava 3.1.10 |
| Escopo de findings | apenas a extensão `.dex` neste caminho exato; nenhum outro binário |
| Validade | 2027-02-05 (três meses); depois disso o arquivo sai ou a exceção é renovada |

Proveniência: `crates/rd-vm/tests/fixtures/rxjava3/README.md` e
`scripts/gen-rxjava3-fixture.sh`, que verifica os hashes das entradas e da saída.
Licenças: RxJava Apache-2.0; reactive-streams MIT-0 (POM).

## Consequências

- O arquivo fica no histórico do git. Remover depois exige reescrita de histórico, por isso o tamanho é mantido em 23 KB.
- Não há código executado no build a partir desse arquivo além dos testes.
- Uma nova biblioteca `dex-passthrough` não herda esta exceção. Cada uma precisa da sua.
