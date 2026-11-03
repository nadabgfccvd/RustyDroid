# ADR 022 — Suporte a XAPK (bundle de splits)

- **Status:** aceito (2026-11-02) — implementado em `crates/rd-apk/src/xapk.rs`
- **Escopo:** `crates/rd-apk` (`Apk::open`, módulo `xapk`), `crates/rd-cli` (`rd inspect`)
- **Relacionados:** objetivos adicionais do dono (`docs/ROADMAP.md`, 2026-11-02); ADR 008 (leitura paginada de ZIP grande); ADR 016 (piso de API).

## Contexto

Muitos apps chegam como XAPK: um ZIP com `manifest.json` e vários APKs (um `base` e os splits de configuração e de feature), além de arquivos OBB de expansão. Hoje o `rd` só entende um APK único. Um XAPK é recusado como ZIP sem `AndroidManifest.xml`.

## Decisão

1. **Detecção:** um ZIP é um bundle quando tem `manifest.json` e não tem `AndroidManifest.xml` na raiz (`xapk::is_bundle`). Um APK comum nunca é tratado como bundle.
2. **Split base:** o split com `id == "base"` em `split_apks` é o APK que `Apk::open` e `Apk::from_bytes` devolvem. Os demais splits são carregados e ficam em `Xapk::splits`. Se não houver split base, a leitura falha com `INVALID_FORMAT`.
3. **Erros estruturados:** `manifest.json` inválido gera `PARSE_ERROR`; um split listado e ausente do ZIP gera `MISSING_ENTRY` com o nome do arquivo.
4. **OBB:** são listados os arquivos de `expansions` presentes no ZIP. Nada é extraído nem copiado.
5. **Inspect:** `rd inspect` mostra um bloco `bundle` (texto) e um objeto `bundle` (`--json`) com o manifesto, os splits (id, arquivo, base, DEX, libs, ABIs) e os OBB.

## Consequências

- O inventário de `content` do `inspect` (contagens de DEX, libs e assets) continua sendo o do split base. Os totais por split aparecem no bloco `bundle`.
- `rd dex disasm` lê os DEX de todos os splits. O split base vem primeiro, porque `--class` usa o primeiro dex que contém a classe. Cada dex leva o id do split como prefixo do nome.
- **Instalação (2026-11-02, #487):** todos os splits são parseados e validados; o split `base` é instalado e a resposta traz `bundle.splits`.
- **DEX de splits na execução (2026-11-03):** o DEX de cada split de feature entra no classpath da execução (`bundle.split_dex_merged`). Teste de agente: `install_xapk_mounts_split_dex_into_execution`.
- **Libs nativas de splits: ficam de fora (base-only), por decisão.** Uma lib nativa em split não é carregada. Isso é declarado, não silencioso: `bundle.native_libs_pending` conta as libs pendentes e `bundle.base_only_execution` fica `true` enquanto houver alguma. Carregar libs de split exige suporte a `System.loadLibrary` por split, que fica para uma ADR própria se for necessário.
- **`rd test` sobre XAPK:** verificado pelo E2E `rd_test_runs_two_scenarios_over_an_xapk_bundle_with_split_dex` (`crates/rd-cli/tests/test_cli.rs`), com 2 cenários: instalar o bundle, lançar, conferir a tela; e reinstalar o mesmo bundle e conferir a mesma tela.
- Um XAPK é lido inteiro na memória, como um APK comum abaixo de 512 MB. Bundles maiores que isso não usam o modo paginado de ADR 008 nesta versão.
- A instalação de XAPK não está coberta por esta decisão.

## Alternativas consideradas

- **Descompactar o XAPK em um diretório antes de abrir:** adiciona escrita em disco e não resolve o problema de identificar o base.
- **Escolher o base pelo nome do arquivo:** frágil. O `id` do manifesto é a fonte de verdade.
