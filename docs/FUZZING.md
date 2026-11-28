# Fuzzing — RustyDroid

Os parsers de formato não-confiável (ZIP, AXML, ARSC, DEX, ELF e WASM) são
fuzz-hardened: toda leitura é bounds-checked, todo loop tem guarda de avanço e
a Lei nº 1 proíbe pânico — falhas são sempre
`RdError { code, cause, suggestion, module_id }` (ou `WasmError` estruturado em
`rd-engine-wasm`).

## Layout

- `crates/rd-apk/fuzz/` — targets `zip`, `axml`, `arsc`, `apk_full`
- `crates/rd-dex/fuzz/` — target `dex` (parse + render das primeiras classes)
- `crates/rd-ndk/fuzz/` — target `elf` (parser ELF32/64 de `.so` não confiável)
- `crates/rd-engine-wasm/fuzz/` — target `wasm_module` (parser + validador + interpretador WASM, M11 S1)

Todos os 7 targets entram na campanha noturna (`.github/workflows/fuzz.yml`), no
job `fuzz-build` do CI e no script `scripts/fuzz-campaign.sh`.

Os crates de fuzz ficam FORA do workspace principal (padrão `cargo-fuzz`) para
não contaminar o MSRV nem o lockfile de produção. Por isso os comandos
`cargo fuzz` devem rodar **dentro da pasta do crate dono de `fuzz/`**, e não na
raiz do repositório.

## Rodando

```bash
cargo install cargo-fuzz
(cd crates/rd-apk && cargo +nightly fuzz build)               # valida os 4 targets de APK
(cd crates/rd-dex && cargo +nightly fuzz build)               # valida o target DEX
(cd crates/rd-ndk && cargo +nightly fuzz build)               # valida o target ELF
(cd crates/rd-engine-wasm && cargo +nightly fuzz build)       # valida o target WASM

(cd crates/rd-apk && cargo +nightly fuzz run zip -- -max_total_time=60)
(cd crates/rd-dex && cargo +nightly fuzz run dex -- -max_total_time=60)
(cd crates/rd-ndk && cargo +nightly fuzz run elf -- -max_total_time=60)
(cd crates/rd-engine-wasm && cargo +nightly fuzz run wasm_module -- -max_total_time=60)
```

## Política

- O DoD do M1 pede fuzz 24 h sem crash antes de declarar o parser "estável".
  O script `scripts/fuzz-campaign.sh` roda os 7 targets em paralelo por 24 h e
  arquiva os crashes (se houver) para abertura de issue com o input minimizado.
- O CI roda `cargo fuzz build` (compilação em nightly, sem execução) para
  garantir que os targets não apodrecem entre milestones.
- Qualquer crash de fuzz é bug P0 — Lei nº 1 violada.
