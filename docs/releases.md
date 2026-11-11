# Releases

Binários do `rd`, SBOM e scoreboards saem do
[`release.yml`](https://github.com/nadabgfccvd/RustyDroid/blob/main/.github/workflows/release.yml)
a cada tag `v*`. Esta página é gerada a partir da API do GitHub no build do
site, então ela lista o que **existe publicado**, não o que o repositório
pretende publicar.

<!-- releases-live:begin -->

**1 release(s)** publicada(s), **0** artefato(s) anexado(s). Tudo gerado pelo `release.yml` a partir da tag.

| versão | publicada em | artefatos | tamanho |
|---|---|---:|---:|
| [`v0.9.0`](https://github.com/nadabgfccvd/RustyDroid/releases/tag/v0.9.0) | 2026-10-19 | 0 | — |

Cada payload traz o binário `rd`, o SBOM CycloneDX, `SHA256SUMS.txt`, a matriz de compatibilidade do commit e a licença. A procedência é atestada via OIDC (`actions/attest-build-provenance`); confira com `gh attestation verify <arquivo> --repo nadabgfccvd/RustyDroid`.

<!-- releases-live:end -->

## O que vem em cada payload

| arquivo | o que é |
|---|---|
| `rustydroid-<tag>-linux-x86_64.tar.gz` | `rd` (build `--release --all-features`) + licenças + `RustyDroid.md` + matriz de compatibilidade do commit |
| `SHA256SUMS.txt` | checksum do tarball |
| `sbom.cdx.json` | SBOM CycloneDX do binário (via `cargo-cyclonedx`) |

Procedência: o tarball recebe uma atestação SLSA assinada por OIDC. Para
verificar localmente:

```bash
gh attestation verify rustydroid-vX.Y.Z-linux-x86_64.tar.gz \
  --repo nadabgfccvd/RustyDroid
```

## Como uma release é cortada

1. `CHANGELOG.md` move `[Unreleased]` para a versão (`GOVERNANCE.md`).
2. `Cargo.toml` (workspace) recebe a mesma versão.
3. `git tag -a vX.Y.Z -m "RustyDroid vX.Y.Z"` e `git push origin vX.Y.Z`.
4. O `release.yml` constrói, **prova que `docs/compat-matrix.md` está em sincronia
   com o commit da tag** (senão falha), assina a procedência e publica.
   Reexecutar com `workflow_dispatch` republica a mesma tag com `--clobber`.
