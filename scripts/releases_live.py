#!/usr/bin/env python3
"""Renderiza a página de releases do site (M12).

O `release.yml` é tag-driven: publica o tarball do `rd`, o SBOM CycloneDX, a
matriz de compatibilidade do commit e a procedência assinada. O que faltava era
**descobrir** isso pelo site — a página de releases era uma lista escrita à mão
que parava na v0.9.0.

O job `build` do `docs.yml` busca `GET /repos/:owner/:repo/releases` e chama

    python3 scripts/releases_live.py --api-json releases.json

que reescreve o bloco entre `releases-live:begin` e `releases-live:end` em
`docs/releases.md`.

Política de falha, diferente da página de compatibilidade: uma lista de releases
publicadas **não envelhece** como um placar de nightly. Se a API não responder,
o script sai com 0 **sem tocar na página** — o build publica o último snapshot
versionado em vez de apagar o histórico. Só uma resposta válida e vazia (nenhuma
release ainda) substitui o bloco.

Uso:
    releases_live.py [--api-json FILE] [--page FILE] [--check] [--quiet]
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys

BEGIN = "<!-- releases-live:begin -->"
END = "<!-- releases-live:end -->"

NO_RELEASES = (
    "> **Nenhuma release publicada ainda.** O `release.yml` publica quando uma "
    "tag `v*` é empurrada; a lista aparece aqui automaticamente depois disso."
)


def human_size(n: int) -> str:
    value = float(n)
    for unit in ("B", "KiB", "MiB", "GiB"):
        if value < 1024 or unit == "GiB":
            return f"{value:.0f} {unit}" if unit == "B" else f"{value:.2f} {unit}"
        value /= 1024
    return f"{n} B"


def load(path: pathlib.Path | None) -> list[dict] | None:
    """`None` = sem dados utilizáveis (não sobrescrever); `[]` = resposta válida vazia."""
    if path is None or not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return None
    if not isinstance(data, list):
        return None
    return [r for r in data if isinstance(r, dict) and r.get("tag_name")]


def render_block(releases: list[dict]) -> str:
    lines = [BEGIN, ""]
    if not releases:
        lines += [NO_RELEASES, "", END]
        return "\n".join(lines)

    total_assets = sum(len(r.get("assets") or []) for r in releases)
    lines.append(
        f"**{len(releases)} release(s)** publicada(s), **{total_assets}** artefato(s) "
        "anexado(s). Tudo gerado pelo `release.yml` a partir da tag."
    )
    lines.append("")
    lines.append("| versão | publicada em | artefatos | tamanho |")
    lines.append("|---|---|---:|---:|")
    for r in releases:
        assets = r.get("assets") or []
        size = sum(int(a.get("size") or 0) for a in assets)
        tag = str(r["tag_name"])
        url = r.get("html_url") or ""
        cell = f"[`{tag}`]({url})" if url else f"`{tag}`"
        date = str(r.get("published_at") or "")[:10] or "—"
        lines.append(
            f"| {cell} | {date} | {len(assets)} | "
            f"{human_size(size) if assets else '—'} |"
        )
    lines.append("")

    for r in releases:
        assets = r.get("assets") or []
        if not assets:
            continue
        tag = str(r["tag_name"])
        lines.append(f"<details><summary><code>{tag}</code> — arquivos</summary>")
        lines.append("")
        lines.append("| arquivo | tamanho |")
        lines.append("|---|---:|")
        for a in sorted(assets, key=lambda a: str(a.get("name"))):
            name = str(a.get("name") or "?")
            link = a.get("browser_download_url") or ""
            cell = f"[`{name}`]({link})" if link else f"`{name}`"
            lines.append(f"| {cell} | {human_size(int(a.get('size') or 0))} |")
        lines.append("")
        lines.append("</details>")
        lines.append("")

    lines.append(
        "Cada payload traz o binário `rd`, o SBOM CycloneDX, `SHA256SUMS.txt`, a "
        "matriz de compatibilidade do commit e a licença. A procedência é atestada "
        "via OIDC (`actions/attest-build-provenance`); confira com "
        "`gh attestation verify <arquivo> --repo nadabgfccvd/RustyDroid`."
    )
    lines.append("")
    lines.append(END)
    return "\n".join(lines)


def replace_block(page_text: str, block: str) -> str:
    start = page_text.find(BEGIN)
    stop = page_text.find(END)
    if start < 0 or stop < 0 or stop < start:
        print(f"erro: marcadores {BEGIN} / {END} ausentes ou fora de ordem", file=sys.stderr)
        raise SystemExit(2)
    return page_text[:start] + block + page_text[stop + len(END):]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--api-json", type=pathlib.Path)
    ap.add_argument("--page", type=pathlib.Path, default=pathlib.Path("docs/releases.md"))
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args(argv)

    releases = load(args.api_json)
    if releases is None:
        if not args.quiet:
            print("releases-live: sem dados da API; página mantida como está")
        return 0

    block = render_block(releases)
    try:
        page_text = args.page.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"erro: não consegui ler {args.page}: {exc}", file=sys.stderr)
        return 2

    if args.check:
        replace_block(page_text, block)
        print(block)
        return 0

    args.page.write_text(replace_block(page_text, block), encoding="utf-8")
    if not args.quiet:
        print(f"releases-live: {len(releases)} release(s) -> {args.page}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
