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
- `rd test` e a instalação ainda operam só sobre o split base. Isso é uma limitação conhecida, não um comportamento silencioso: está documentada no módulo, no CHANGELOG e no ROADMAP.
- Um XAPK é lido inteiro na memória, como um APK comum abaixo de 512 MB. Bundles maiores que isso não usam o modo paginado de ADR 008 nesta versão.
- A instalação de XAPK não está coberta por esta decisão.

## Alternativas consideradas

- **Descompactar o XAPK em um diretório antes de abrir:** adiciona escrita em disco e não resolve o problema de identificar o base.
- **Escolher o base pelo nome do arquivo:** frágil. O `id` do manifesto é a fonte de verdade.
