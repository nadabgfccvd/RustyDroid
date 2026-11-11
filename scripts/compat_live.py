#!/usr/bin/env python3
"""Renderiza o bloco "vivo" da matriz de compatibilidade (M12).

`nightly-compat.yml` roda a suíte de APKs dourados todo dia e publica
**artefatos** (JUnit XML por suíte, JSON de `rd inspect`, o scoreboard gerado).
Quem publica no GH Pages é `docs.yml` — um único escritor na branch `gh-pages`,
o que evita corrida com o `bench.yml` (que guarda `dev/bench/` lá).

Este script é a ponte entre os dois: o job `build` do `docs.yml` baixa o
artefato da última run noturna e chama

    python3 scripts/compat_live.py --reports <dir> --run-json <run.json>

que reescreve o bloco entre `compat-live:begin` / `compat-live:end` em
`docs/compat-live.md` e regenera `docs/compat-badge.svg`. O `mdbook build` que
vem depois publica os dois. Sem dados (primeira execução, run falhou, artefato
expirado) o bloco volta ao estado "sem dados" — a página continua válida.

Uso:
    compat_live.py [--reports DIR] [--run-json FILE] [--page FILE]
                   [--badge FILE] [--check] [--quiet]

`--check` só valida que a página tem os marcadores e imprime o render em
stdout; é o que o self-test usa.

Determinismo: nada de `now()`. O único carimbo de tempo é o `created_at` da
run, então rodar o script duas vezes com os mesmos dados gera o mesmo byte a
byte — dá para usar `git diff` como gate.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
import xml.etree.ElementTree as ET

BEGIN = "<!-- compat-live:begin -->"
END = "<!-- compat-live:end -->"

NO_DATA = (
    "> **Sem dados recentes.** Nenhuma execução do `nightly-compat.yml` foi "
    "encontrada para esta publicação. O placar abaixo da página é o último "
    "versionado (`docs/compat-matrix.md`), mas a tabela por suíte só aparece "
    "depois da primeira run noturna bem-sucedida."
)


def _int(value: str | None) -> int:
    try:
        return int(float(value or 0))
    except (TypeError, ValueError):
        return 0


def read_junit(path: pathlib.Path) -> dict[str, object] | None:
    """Lê um JUnit XML do `rd test --junit` e devolve os contadores da suíte.

    Aceita raiz `<testsuites>` (o que o `rd` escreve) ou `<testsuite>` solto.
    Se os atributos não existirem, conta os `<testcase>` filhos.
    """
    try:
        root = ET.parse(path).getroot()
    except (ET.ParseError, OSError):
        return None
    suites = [root] if root.tag == "testsuite" else list(root.iter("testsuite"))
    if not suites:
        return None
    tests = sum(_int(s.get("tests")) for s in suites)
    failures = sum(_int(s.get("failures")) for s in suites)
    errors = sum(_int(s.get("errors")) for s in suites)
    skipped = sum(_int(s.get("skipped")) for s in suites)
    seconds = sum(float(s.get("time") or 0.0) for s in suites)
    if tests == 0 and not any(s.get("tests") for s in suites):
        tests = sum(len(list(s.iter("testcase"))) for s in suites)
    name = (suites[0].get("name") or path.stem).strip()
    return {
        "suite": name,
        "file": path.name,
        "tests": tests,
        "failures": failures,
        "errors": errors,
        "skipped": skipped,
        "seconds": seconds,
    }


def collect(reports: pathlib.Path | None) -> list[dict[str, object]]:
    if reports is None:
        return []
    junit_dir = reports / "junit"
    if not junit_dir.is_dir():
        return []
    rows = []
    for xml in sorted(junit_dir.glob("*.xml")):
        row = read_junit(xml)
        if row is not None:
            rows.append(row)
    rows.sort(key=lambda r: str(r["suite"]))
    return rows


def count_inspect(reports: pathlib.Path | None) -> int:
    if reports is None:
        return 0
    d = reports / "inspect"
    return len(list(d.glob("*.json"))) if d.is_dir() else 0


def read_run(path: pathlib.Path | None) -> dict[str, object]:
    if path is None or not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {}
    return data if isinstance(data, dict) else {}


def _fmt_seconds(total: float) -> str:
    if total < 1:
        return f"{total * 1000:.0f} ms"
    return f"{total:.2f} s"


def verdict(rows: list[dict[str, object]], run: dict[str, object]) -> tuple[str, str, str]:
    """(símbolo, texto do badge, cor) a partir dos dados disponíveis."""
    if not rows:
        return "⚪", "sem dados", "#9e9e9e"
    bad = sum(int(r["failures"]) + int(r["errors"]) for r in rows)
    conclusion = str(run.get("conclusion") or "")
    if bad or conclusion in {"failure", "timed_out", "cancelled"}:
        return "❌", f"{bad} falha(s)", "#c62828"
    return "✅", "verde", "#2e7d32"


def render_block(rows: list[dict[str, object]], inspect_count: int, run: dict[str, object]) -> str:
    symbol, text, _color = verdict(rows, run)
    lines: list[str] = []
    lines.append(BEGIN)
    lines.append("")
    if not rows:
        lines.append(NO_DATA)
        lines.append("")
        lines.append(END)
        return "\n".join(lines)

    total = sum(int(r["tests"]) for r in rows)
    bad = sum(int(r["failures"]) + int(r["errors"]) for r in rows)
    skipped = sum(int(r["skipped"]) for r in rows)
    seconds = sum(float(r["seconds"]) for r in rows)

    head = f"**Status da última execução noturna: {symbol} {text}**"
    lines.append(head)
    lines.append("")

    meta: list[str] = []
    if run.get("created_at"):
        meta.append(f"execução de `{run['created_at']}`")
    if run.get("head_sha"):
        meta.append(f"commit `{str(run['head_sha'])[:7]}`")
    if run.get("run_number"):
        meta.append(f"run #{run['run_number']}")
    if meta:
        lines.append(" · ".join(meta))
        lines.append("")
    if run.get("html_url"):
        lines.append(f"[Log da execução no GitHub Actions]({run['html_url']})")
        lines.append("")

    lines.append("| suíte | testes | falhas | erros | pulados | tempo |")
    lines.append("|---|---:|---:|---:|---:|---:|")
    for r in rows:
        name = str(r["suite"])
        if int(r["failures"]) + int(r["errors"]) > 0:
            name = f"**{name}** ❌"
        lines.append(
            f"| {name} | {r['tests']} | {r['failures']} | {r['errors']} | "
            f"{r['skipped']} | {_fmt_seconds(float(r['seconds']))} |"
        )
    lines.append(
        f"| **total** | **{total}** | **{bad}** | 0 | {skipped} | "
        f"**{_fmt_seconds(seconds)}** |"
    )
    lines.append("")
    if inspect_count:
        lines.append(
            f"Também nesta rodada: **{inspect_count}** APK(s) inspecionado(s) em "
            "JSON estruturado (artefato `nightly-compat-reports`, retenção de 14 dias)."
        )
        lines.append("")
    lines.append(
        "A tabela é gerada por `scripts/compat_live.py` a partir do JUnit XML do "
        "`rd test`; o placar oficial por API continua em "
        "[compat-matrix](./compat-matrix.md)."
    )
    lines.append("")
    lines.append(END)
    return "\n".join(lines)


def render_badge(rows: list[dict[str, object]], run: dict[str, object]) -> str:
    _symbol, text, color = verdict(rows, run)
    if rows:
        total = sum(int(r["tests"]) for r in rows)
        right = f"{total} testes · {text}"
    else:
        right = text
    left_w, right_w = 108, max(90, 8 + 7 * len(right))
    total_w = left_w + right_w
    title = f"compat nightly: {right}"
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}" height="20" role="img" aria-label="{title}">
  <title>{title}</title>
  <linearGradient id="s" x2="0" y2="100%">
    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>
    <stop offset="1" stop-opacity=".1"/>
  </linearGradient>
  <rect width="{total_w}" height="20" rx="3" fill="#555"/>
  <rect x="{left_w}" width="{right_w}" height="20" rx="3" fill="{color}"/>
  <rect width="{total_w}" height="20" rx="3" fill="url(#s)"/>
  <g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11">
    <text x="{left_w // 2}" y="14" fill="#010101" fill-opacity=".3">compat nightly</text>
    <text x="{left_w // 2}" y="13">compat nightly</text>
    <text x="{left_w + right_w // 2}" y="14" fill="#010101" fill-opacity=".3">{right}</text>
    <text x="{left_w + right_w // 2}" y="13">{right}</text>
  </g>
</svg>
"""


def replace_block(page_text: str, block: str) -> str:
    start = page_text.find(BEGIN)
    stop = page_text.find(END)
    if start < 0 or stop < 0 or stop < start:
        raise SystemExit(
            f"erro: marcadores {BEGIN} / {END} ausentes ou fora de ordem na página"
        )
    return page_text[:start] + block + page_text[stop + len(END):]


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--reports", type=pathlib.Path, help="diretório do artefato (junit/, inspect/)")
    ap.add_argument("--run-json", type=pathlib.Path, help="JSON da workflow run (API do GitHub)")
    ap.add_argument("--page", type=pathlib.Path, default=pathlib.Path("docs/compat-live.md"))
    ap.add_argument("--badge", type=pathlib.Path, default=pathlib.Path("docs/compat-badge.svg"))
    ap.add_argument("--check", action="store_true", help="não escreve; imprime o bloco")
    ap.add_argument("--quiet", action="store_true")
    args = ap.parse_args(argv)

    rows = collect(args.reports)
    inspect_count = count_inspect(args.reports)
    run = read_run(args.run_json)
    block = render_block(rows, inspect_count, run)

    if args.check:
        if not args.page.is_file():
            print(f"erro: página não encontrada: {args.page}", file=sys.stderr)
            return 2
        replace_block(args.page.read_text(encoding="utf-8"), block)  # valida marcadores
        print(block)
        return 0

    try:
        page_text = args.page.read_text(encoding="utf-8")
    except OSError as exc:
        print(f"erro: não consegui ler {args.page}: {exc}", file=sys.stderr)
        return 2
    args.page.write_text(replace_block(page_text, block), encoding="utf-8")
    args.badge.write_text(render_badge(rows, run), encoding="utf-8")

    if not args.quiet:
        _symbol, text, _color = verdict(rows, run)
        suites = len(rows)
        tests = sum(int(r["tests"]) for r in rows)
        print(
            f"compat-live: {suites} suíte(s), {tests} teste(s), status {text} -> {args.page}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
