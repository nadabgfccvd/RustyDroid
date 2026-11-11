#!/usr/bin/env python3
"""Reescreve, no HTML gerado, os links que escapam da raiz do livro.

## O problema

`docs/` é um livro mdBook, mas também é um diretório do repositório. As duas
visões não concordam sobre `../`:

* **visão do repositório** — `docs/adr/021-apk-layering.md` linkando
  `../README.md` aponta para um arquivo real, e `scripts/check_md_links.py`
  valida isso corretamente;
* **visão do livro** — o mdBook só renderiza o que está sob `docs/`, então o
  mesmo link vira `/README.html`, que **não existe** no site publicado. São 42
  páginas nessa situação hoje, e nove delas na página inicial (que era o
  primeiro coisa que um visitante clicava).

Corrigir os fontes quebraria a visão do repositório. Então a correção é feita
onde ela é verdadeira: no HTML, depois do build.

## O segundo problema (o `README.md` de uma subpasta)

O mdBook renderiza `docs/arquivo/README.md` como `arquivo/index.html`, **não**
como `arquivo/README.html`. Um link escrito `./archive/README.md` — correto na
visão do repositório, onde o arquivo existe — vira `archive/README.html` no
HTML e responde **404** no site. Foi assim que `docs/index.md` passou a linkar
um 404 em `archive/README.html` (achado no crawl de 2026-11-10).

Aqui também a correção é no HTML: se o alvo `.../README.html` não existe mas
`.../index.html` existe, reescreve. Sem alvo nenhum, vira "leftover" e o script
sai com 1, para o build falhar em vez de publicar link morto.

## O que faz

Troca `href="../X"` por
`https://github.com/<repo>/blob/main/X`, resolvendo os `../` a partir da
profundidade da página. Idempotente: rodar duas vezes não muda nada.

Uso:
  book_links.py --site target/mdbook [--repo nadabgfccvd/RustyDroid] [--ref main]

Exit 0 sempre que sobrou zero link escapando; 1 se sobrou algum.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

_HREF = re.compile(r'(?P<attr>\b(?:href|src))="(?P<target>\.\.(?:/[^"]*)?)"')
# Alvo relativo terminando em `README.html`: no mdBook a página real é `index.html`.
_README = re.compile(r'(?P<attr>\b(?:href|src))="(?P<target>(?![a-z]+:)[^"#]*README\.html)"')
_SHALLOW = re.compile(r'^\.\./(?:(?:\.\./)+)?(?P<rest>.*)$')


def rewrite(html: str, page: Path, site: Path, blob: str) -> tuple[str, int, list[str]]:
    """Reescreve só os alvos que **não existem** dentro do site.

    Verificar a existência é obrigatório: numa página em `adr/`, o mdBook emite
    `href="../css/variables-*.css"` e `href="../favicon-*.svg"`, que são
    **válidos** — resolvem para `/css/...` e `/favicon-...` na raiz do site.
    Uma regex ingênua sobre `../` transformaria 918 assets legítimos em links
    externos.

    Devolve `(html, reescritos, alvos_ausentes)`. `re.subn` **não** serve para
    contar: ele conta matches, incluindo os que a função devolve intactos.
    """
    site_abs = site.resolve()
    changed = 0
    missing: list[str] = []

    def repl(m: re.Match[str]) -> str:
        nonlocal changed
        target = m.group("target")
        resolved = (page.parent / target).resolve()
        inside = True
        try:
            resolved.relative_to(site_abs)
        except ValueError:
            inside = False
        if inside and resolved.exists():
            return m.group(0)  # asset do próprio livro: preservar
        missing.append(target)
        rest = target
        while rest.startswith("../"):
            rest = rest[3:]
        if not rest:
            return m.group(0)
        changed += 1
        return f'{m.group("attr")}="{blob}/{rest}"'

    return _HREF.sub(repl, html), changed, missing


def readme_to_index(html: str, page: Path, site: Path) -> tuple[str, int, list[str]]:
    """Reescreve `X/README.html` -> `X/index.html` quando só o segundo existe.

    Devolve `(html, reescritos, alvos_ausentes)`, com a mesma convenção de
    `rewrite`: alvo ausente sem `index.html` substituto entra em `alvos_ausentes`
    (o chamador transforma isso em falha).
    """
    site_abs = site.resolve()
    changed = 0
    missing: list[str] = []

    def repl(m: re.Match[str]) -> str:
        nonlocal changed
        target = m.group("target")
        if (page.parent / target).exists():
            return m.group(0)
        alt = target[: -len("README.html")] + "index.html"
        alt_path = (page.parent / alt).resolve()
        inside = True
        try:
            alt_path.relative_to(site_abs)
        except ValueError:
            inside = False
        if inside and alt_path.exists():
            changed += 1
            return f'{m.group("attr")}="{alt}"'
        missing.append(target)
        return m.group(0)

    return _README.sub(repl, html), changed, missing


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--site", required=True)
    ap.add_argument("--repo", default="nadabgfccvd/RustyDroid")
    ap.add_argument("--ref", default="main")
    ap.add_argument("--check", action="store_true", help="não escreve; só verifica")
    args = ap.parse_args(argv)

    site = Path(args.site)
    if not site.is_dir():
        print(f"book_links: {site} is not a directory", file=sys.stderr)
        return 2

    blob = f"https://github.com/{args.repo}/blob/{args.ref}"
    pages = 0
    rewrites = 0
    leftovers: list[str] = []

    for page in sorted(site.rglob("*.html")):
        html = page.read_text(encoding="utf-8")
        new, n, missing = rewrite(html, page, site, blob)
        new, n2, missing2 = readme_to_index(new, page, site)
        if n or n2:
            pages += 1
            rewrites += n + n2
            if not args.check:
                page.write_text(new, encoding="utf-8")
        # Depois de reescrever (ou não, em --check), nenhum alvo pode continuar
        # ausente: é isso que quebra na visão do livro.
        _, _, still_missing = rewrite(new, page, site, blob)
        _, _, still_missing2 = readme_to_index(new, page, site)
        if still_missing or still_missing2:
            alvos = still_missing[:3] + still_missing2[:3]
            leftovers.append(f"{page.relative_to(site)} -> {', '.join(alvos)}")

    verb = "verificaria" if args.check else "reescreveu"
    print(f"book_links: {verb} {rewrites} link(s) em {pages} página(s) -> {blob}")

    if leftovers:
        print(
            f"book_links: {len(leftovers)} página(s) ainda linkam para alvo inexistente: "
            + ", ".join(leftovers[:8]),
            file=sys.stderr,
        )
        return 1

    print("book_links: nenhum link escapa da raiz do livro")
    return 0


if __name__ == "__main__":
    sys.exit(main())
