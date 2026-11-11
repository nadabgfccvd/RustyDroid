# Compatibilidade ao vivo (última rodada noturna)

![compat nightly](./compat-badge.svg)

Esta página é o **placar vivo**: o resultado da última execução do
[`nightly-compat.yml`](https://github.com/nadabgfccvd/RustyDroid/blob/main/.github/workflows/nightly-compat.yml)
sobre os APKs dourados, publicado junto com o site.

- O nightly roda todo dia às **04:43 UTC** (ou por `workflow_dispatch`), inspeciona
  cada APK dourado, faz o diff de disassembly contra o baksmali e executa todas as
  suítes de `tests/suites/` em JUnit XML.
- O job `build` do `docs.yml` baixa o artefato `nightly-compat-reports` da última
  run **bem-sucedida**, e `scripts/compat_live.py` reescreve o bloco abaixo e o
  badge acima antes do `mdbook build`. Publicar continua sendo só do `docs.yml`
  (um escritor na `gh-pages`, sem corrida com o `bench.yml`).
- Se não houver run recente, a página diz isso em vez de inventar números — a
  matriz oficial por API, que não depende de APK, continua em
  [compat-matrix](./compat-matrix.md).

O que **não** está aqui: medições de performance (essas ficam no gráfico
`dev/bench/` do site e no gate da
[ADR 038](./adr/038-gate-de-benchmark-invariante-ao-runner.md)) e o histórico de
versões, que é o [CHANGELOG](https://github.com/nadabgfccvd/RustyDroid/blob/main/CHANGELOG.md).

<!-- compat-live:begin -->

> **Sem dados recentes.** Nenhuma execução do `nightly-compat.yml` foi encontrada
> para esta publicação. O placar abaixo da página é o último versionado
> (`docs/compat-matrix.md`), mas a tabela por suíte só aparece depois da primeira
> run noturna bem-sucedida.

<!-- compat-live:end -->

## Como reproduzir localmente

```bash
cargo build --release -p rd-cli
./golden-apks/fetch.sh org.schabi.newpipe org.tasks
mkdir -p reports/junit
for suite in tests/suites/*.json; do
  ./target/release/rd test "$suite" --junit "reports/junit/$(basename "$suite" .json).xml"
done
python3 scripts/compat_live.py --reports reports --page docs/compat-live.md
```

Sem `--run-json` a página fica com os contadores das suítes e sem o commit de
origem — é o modo útil para um desenvolvedor checar antes de abrir PR.
