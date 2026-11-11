#!/usr/bin/env python3
"""Gate de regressão por **contagem de instruções**, não por tempo de parede.

## Por que o wall-clock não serve (histórico, Issue #629)

A §3.2 da spec pede "fail em regressão > 10%" no `bench.yml`. A primeira
implementação comparava o tempo com a execução anterior. Medido no parque real
de runners, três execuções consecutivas do **mesmo código** deram:

    run   calib_scan   parse_method   razão (parse/calib)
    731      43 164        3 196          0,0740
    732      68 506        3 234          0,0472
    733      42 310        3 160          0,0747
    espalhamento 62%          2,3%           58%

Ou seja: o alvo é estável (2,3%) e a *calibração* é a grandeza ruidosa (62%) —
a tentativa de normalizar deixou a métrica 5× pior que o valor bruto. A série
histórica inteira (191 amostras) vai de 1 736 a 17 370 ns por iteração, com um
vazio entre 1,25× e 2× a mediana: são classes de CPU distintas. Nenhuma
estatística conserta isso, porque o mensurando é a frota.

## A métrica

Instruções retiradas (`callgrind`). Determinístico: três execuções do mesmo
binário dão o **mesmo número**, sem variância:

    calib_scan_20k      1 898 670
    parse_method_ids     923 930
    parse_field_ids      923 958

O custo fixo do processo (startup + montagem do buffer) é **95% da contagem**
(878 378 de 923 930), então o gate subtrai a linha `noop` antes de comparar.
Sem isso um limiar de 10% incidiria sobre o runtime, não sobre o trabalho.

O que isso deixa passar, declaradamente: uma mudança que troca instruções por
ciclos (pior IPC) aparece como melhoria. Por isso os nanossegundos continuam
sendo medidos e publicados como **advisory** no gráfico do `gh-pages`.

## Baseline

`crates/rd-dex/benches/ir-baseline.json` é versionado. Como a contagem é
determinística, não há histórico a acumular: quando uma mudança altera o
trabalho legitimamente, o PR atualiza o baseline e a revisão vê o diff.

    # compara (falha se > 10% pior)
    python3 scripts/bench_ir.py --results ir.txt
    # regenera o baseline a partir da medição local
    python3 scripts/bench_ir.py --results ir.txt --write

## Entrada

`--results` recebe linhas `ir <nome> <contagem>`, produzidas pelo workflow a
partir do `callgrind_annotate`. `scripts/bench_ir.py --from-callgrind` ajuda a
montar o arquivo a partir dos `callgrind.out.*`.

Exit 0 dentro do limiar, 1 em regressão, 2 em erro de uso/IO.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "crates" / "rd-dex" / "benches" / "ir-baseline.json"

#: Linha de base do processo: nunca é comparada, só subtraída.
NOOP = "noop"


def parse_results(text: str) -> dict[str, int]:
    """`ir <nome> <contagem>` por linha -> {nome: contagem}."""
    out: dict[str, int] = {}
    for lineno, line in enumerate(text.splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) != 3 or parts[0] != "ir":
            raise ValueError(f"linha {lineno}: esperado `ir <nome> <contagem>`, veio {line!r}")
        try:
            out[parts[1]] = int(parts[2])
        except ValueError:
            raise ValueError(f"linha {lineno}: contagem não numérica em {line!r}") from None
    return out


def work_counts(counts: dict[str, int]) -> dict[str, int]:
    """Subtrai o custo fixo do processo. Sem `noop`, devolve como está."""
    overhead = counts.get(NOOP, 0)
    return {
        name: value - overhead
        for name, value in counts.items()
        if name != NOOP and value - overhead > 0
    }


def load_baseline(path: Path) -> dict[str, int]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("baseline não é um objeto JSON")
    return {k: int(v) for k, v in data.items() if isinstance(v, (int, float))}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="Gate de instruções (ADR 038).")
    ap.add_argument("--results", required=True, help="arquivo `ir <nome> <contagem>`")
    ap.add_argument("--baseline", default=str(BASELINE))
    ap.add_argument("--threshold", type=float, default=0.10)
    ap.add_argument("--write", action="store_true", help="regrava o baseline")
    ap.add_argument(
        "--from-callgrind",
        nargs="+",
        metavar="ARQ",
        help="monta o --results a partir de callgrind.out.* e sai",
    )
    args = ap.parse_args(argv)

    if args.threshold < 0:
        print("bench_ir: --threshold must be >= 0", file=sys.stderr)
        return 2

    if args.from_callgrind:
        return _from_callgrind(args.from_callgrind, args.results)

    try:
        counts = parse_results(Path(args.results).read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        print(f"bench_ir: cannot read results: {e}", file=sys.stderr)
        return 2
    if not counts:
        print("bench_ir: no `ir` lines in the results file", file=sys.stderr)
        return 2

    work = work_counts(counts)
    if not work:
        print("bench_ir: no countable work after removing the noop overhead", file=sys.stderr)
        return 2

    if args.write:
        ordered = dict(sorted(work.items()))
        Path(args.baseline).write_text(json.dumps(ordered, indent=2) + "\n", encoding="utf-8")
        print(f"bench_ir: wrote {args.baseline} ({len(ordered)} bench(es))")
        return 0

    baseline_path = Path(args.baseline)
    if not baseline_path.exists():
        print(
            f"bench_ir: no baseline at {baseline_path} — run with --write to create it "
            "(expected right after this lands; the gate seeds instead of failing)",
            file=sys.stderr,
        )
        return 0

    try:
        base = load_baseline(baseline_path)
    except (OSError, ValueError) as e:
        print(f"bench_ir: cannot read baseline: {e}", file=sys.stderr)
        return 2

    overhead = counts.get(NOOP, 0)
    print(
        f"bench_ir: instruções retiradas (callgrind) — limiar "
        f"{args.threshold * 100:.0f}%; custo fixo descontado: {overhead:,}"
    )
    print(f"{'benchmark':<28}{'atual':>12}{'baseline':>12}{'ratio':>8}  veredito")

    failures: list[str] = []
    missing: list[str] = []
    for name in sorted(work):
        cur = work[name]
        if name not in base:
            print(f"{name:<28}{cur:>12,}{'—':>12}{'—':>8}  sem baseline")
            missing.append(name)
            continue
        ref = base[name]
        ratio = cur / ref if ref else 0.0
        bad = ratio > 1.0 + args.threshold
        if bad:
            failures.append(name)
        print(f"{name:<28}{cur:>12,}{ref:>12,}{ratio:>8.2f}  {'REGRESSÃO' if bad else 'ok'}")

    if missing and not failures:
        print(
            f"\nbench_ir: {len(missing)} bench(es) sem baseline "
            f"({', '.join(missing)}) — rode `--write` para semear"
        )

    if failures:
        print(
            f"\nbench_ir: {len(failures)} benchmark(s) com mais de "
            f"{args.threshold * 100:.0f}% de instruções: {', '.join(failures)}",
            file=sys.stderr,
        )
        print(
            "bench_ir: se a mudança é intencional, atualize o baseline com "
            "`python3 scripts/bench_ir.py --results <arquivo> --write` e revise o diff "
            "de crates/rd-dex/benches/ir-baseline.json junto com o código.",
            file=sys.stderr,
        )
        return 1

    print("\nbench_ir: todas as contagens dentro do limiar")
    return 0


def _from_callgrind(paths: list[str], out_path: str) -> int:
    """`callgrind.out.*` -> arquivo de resultados. Usa callgrind_annotate."""
    import re
    import subprocess

    lines: list[str] = []
    ir_total = re.compile(r"^([\d,]+)\s", re.M)
    for path in paths:
        m = re.search(r"callgrind\.out\.[^.]*\.?(?P<name>[\w-]+)?$", path)
        name = m.group("name") if m and m.group("name") else Path(path).name
        try:
            done = subprocess.run(
                ["callgrind_annotate", path],
                capture_output=True,
                text=True,
                check=True,
            ).stdout
        except (OSError, subprocess.CalledProcessError) as e:
            print(f"bench_ir: callgrind_annotate failed on {path}: {e}", file=sys.stderr)
            return 2
        hit = ir_total.search(done)
        if not hit:
            print(f"bench_ir: no instruction total in {path}", file=sys.stderr)
            return 2
        lines.append(f"ir {name} {hit.group(1).replace(',', '')}")
    Path(out_path).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"bench_ir: wrote {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
