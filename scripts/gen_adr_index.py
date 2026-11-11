#!/usr/bin/env python3
"""Generate the ADR index so it cannot drift from `docs/adr/`.

`docs/SUMMARY.md` carried a hand-written ADR list that stopped at ADR 020: 18
decision records existed on disk and were invisible in the published book, and
the list was not even in numeric order. A hand-maintained index is a copy of the
directory that nothing checks, so it rots.

This script owns two derived artifacts:

  docs/adr/README.md   — landing page: number, title, date, status per ADR.
  docs/SUMMARY.md      — the block between `<!-- adr:begin -->` and
                         `<!-- adr:end -->`, rewritten in place.

Both are deterministic functions of `docs/adr/*.md`, so CI can regenerate them
and diff, exactly like `cargo xtask compat-report` does for the scoreboard.

Usage:
  gen_adr_index.py            # --check: regenerate in memory, diff, exit 1 on drift
  gen_adr_index.py --write    # rewrite both files
"""

from __future__ import annotations

import argparse
import difflib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ADR_DIR = ROOT / "docs" / "adr"
SUMMARY = ROOT / "docs" / "SUMMARY.md"
README = ADR_DIR / "README.md"

BEGIN = "<!-- adr:begin -->"
END = "<!-- adr:end -->"

_H1 = re.compile(r"^#\s+ADR\s+(?P<num>\d+)\s*[—:–-]\s*(?P<title>.+?)\s*$")

# The header block of an ADR has accumulated four shapes over 39 records:
#   - **Status:** aceito · **Data:** 2026-11-03
#   **Status:** accepted (Comprix e2e, round 3)
#   Status: implemented; ...
#   Issue: #84 ... · Status: aceito · Data: 2026-10-11     (all on one line)
# Rather than normalise 39 files (which would rewrite history), read all four.
_STATUS_PATTERNS = (
    re.compile(r"^\s*-\s+\*\*Status:\*\*\s*(?P<value>.+?)\s*$", re.M),
    re.compile(r"^\s*\*\*Status:\*\*\s*(?P<value>.+?)\s*$", re.M),
    re.compile(r"^\s*Status:\s*(?P<value>.+?)\s*$", re.M),
    re.compile(r"Status:\s*(?P<value>[^·]+)"),
    # a few records use a `## Status` section instead of a header line
    re.compile(r"^##\s+Status\s*\n+(?P<value>.+?)\s*$", re.M),
)
_DATE_PATTERNS = (
    re.compile(r"\*\*(?:Data|Date):\*\*\s*(?P<value>\d{4}-\d{2}-\d{2})"),
    re.compile(r"\b(?:Data|Date):\s*(?P<value>\d{4}-\d{2}-\d{2})"),
)


def collect() -> list[dict[str, str]]:
    """Every `docs/adr/*.md` except the generated README, in numeric order."""
    out: list[dict[str, str]] = []
    for path in sorted(ADR_DIR.glob("*.md")):
        if path.name == "README.md":
            continue
        text = path.read_text(encoding="utf-8")
        first = text.splitlines()[0] if text else ""
        m = _H1.match(first)
        if not m:
            raise SystemExit(f"gen_adr_index: {path.name}: first line is not `# ADR NNN — Title`")
        num = m.group("num")
        # Only the header block (before the first `##` section) can declare status.
        header = text.split("\n## ", 1)[0]
        status = ""
        for pat in _STATUS_PATTERNS:
            ms = pat.search(header if r"##\s+Status" not in pat.pattern else text)
            if ms:
                # keep the first clause only: statuses are free-form prose
                status = re.split(r"\s+[·|]\s+|\s+—\s+|\s+\(", ms.group("value"))[0].rstrip(" .;")
                if len(status) > 56:
                    status = status[:56].rstrip() + "…"
                break
        md = None
        for pat in _DATE_PATTERNS:
            md = pat.search(header)
            if md:
                break
        out.append(
            {
                "num": num,
                "title": m.group("title"),
                "status": status or "—",
                "date": md.group("value") if md else "—",
                "file": path.name,
            }
        )
    out.sort(key=lambda a: (int(a["num"]), a["file"]))
    return out


def render_summary_block(entries: list[dict[str, str]]) -> str:
    lines = [
        BEGIN,
        "- [Índice de ADRs](./adr/README.md)",
    ]
    for e in entries:
        lines.append(f"- [ADR {e['num']} — {e['title']}](./adr/{e['file']})")
    lines.append(END)
    return "\n".join(lines)


def render_readme(entries: list[dict[str, str]]) -> str:
    rows = [
        "| ADR | Decisão | Data | Status |",
        "| --- | --- | --- | --- |",
    ]
    for e in entries:
        rows.append(
            f"| [{e['num']}]({e['file']}) | {e['title']} | {e['date']} | {e['status']} |"
        )
    return (
        "# Índice de Architecture Decision Records\n"
        "\n"
        "<!-- GERADO por scripts/gen_adr_index.py a partir de docs/adr/*.md.\n"
        "     Não edite à mão: rode `python3 scripts/gen_adr_index.py --write`. -->\n"
        "\n"
        f"{len(entries)} registros de decisão. Cada ADR é imutável; uma decisão nova\n"
        "que revisa uma anterior entra como ADR novo que a referencia.\n"
        "\n"
        + "\n".join(rows)
        + "\n"
    )


def apply_summary(text: str, block: str) -> str:
    if BEGIN not in text or END not in text:
        raise SystemExit(
            f"gen_adr_index: {SUMMARY.name} has no {BEGIN} / {END} markers; "
            "add them where the ADR list should go"
        )
    head, rest = text.split(BEGIN, 1)
    _, tail = rest.split(END, 1)
    return f"{head}{block}{tail}"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="rewrite the files in place")
    args = ap.parse_args(argv)

    entries = collect()
    if not entries:
        print(f"gen_adr_index: no ADRs found under {ADR_DIR}", file=sys.stderr)
        return 2

    new_summary = apply_summary(SUMMARY.read_text(encoding="utf-8"), render_summary_block(entries))
    new_readme = render_readme(entries)

    stale: list[tuple[Path, str]] = []
    for path, new in ((SUMMARY, new_summary), (README, new_readme)):
        old = path.read_text(encoding="utf-8") if path.exists() else ""
        if old != new:
            stale.append((path, new))

    if args.write:
        for path, new in stale:
            path.write_text(new, encoding="utf-8")
            print(f"gen_adr_index: wrote {path.relative_to(ROOT)}")
        if not stale:
            print(f"gen_adr_index: {len(entries)} ADRs, already up to date")
        return 0

    if stale:
        for path, new in stale:
            old = path.read_text(encoding="utf-8") if path.exists() else ""
            sys.stdout.writelines(
                difflib.unified_diff(
                    old.splitlines(keepends=True),
                    new.splitlines(keepends=True),
                    fromfile=f"a/{path.relative_to(ROOT)}",
                    tofile=f"b/{path.relative_to(ROOT)}",
                )
            )
        print(
            "\ngen_adr_index: the ADR index is stale — run "
            "`python3 scripts/gen_adr_index.py --write`",
            file=sys.stderr,
        )
        return 1

    print(f"gen_adr_index: {len(entries)} ADRs, index up to date")
    return 0


if __name__ == "__main__":
    sys.exit(main())
