#!/usr/bin/env python3
"""Regression gate for `bench.yml` that is invariant to the runner class.

## Why this exists (Issue #629)

`github-action-benchmark` compares each run against the *single* previous stored
result. That is not a usable oracle on the shared `ubuntu-latest` pool. The
stored series for `rd-dex parse tables` is bimodal: the same commit measures
anywhere from ~1.7 µs to ~17.4 µs per iteration, with an empty band between
1.25x and 2x the median and a *stable* cluster at 5.1x. Those are different
CPU classes, not noise — each class is internally reproducible to ~1%.

So the failure mode is not "the threshold is too tight", it is "the metric
measures the machine fleet". Statistics cannot fix it: replaying the 188 stored
results, a median-of-5 baseline at a 10% threshold still produced **21% false
positives** and **missed ~40% of real 19% regressions**. Widening the window
barely helped (14.4% FP at window 21).

## The fix

Normalise before comparing. The bench emits `calib_scan_20k` — a fixed
sequential scan over the same buffer with the same access pattern — and this
script compares `target / calibration` instead of raw nanoseconds. Measured on
the sandbox: raw time moved **8.39x** (3 867 → 32 451 ns) under CPU contention
while the normalised ratio stayed within 0.0601–0.0661, and the run-to-run
spread dropped from 6.2% to 3.0%.

On top of that the baseline is the **median of the last `--window` stored
results** (default 5), so a single bad run cannot fire the gate. With ~3% ratio
noise the standard error of a median-of-5 is ~1.7%, which puts the 10% rule
required by `RustyDroid.md` §3.2 roughly 6 sigma out.

## Usage

    bench_gate.py --results bench-output.txt --history dev/bench/data.js
                  [--dataset "rd-dex parse tables"]
                  [--reference calib_scan_20k] [--window 5]
                  [--threshold 0.10] [--min-samples 3]

`--reference ''` disables normalisation and falls back to raw nanoseconds
(useful only for debugging).

`--history` may point at a missing file (first run, or a shallow clone without
`gh-pages`). History entries that predate the calibration bench are ignored,
so right after this lands the gate reports that it is **seeding** and passes
until `--min-samples` calibrated results exist.

Exit codes: 0 when every benchmark is within the threshold, 1 on a regression,
2 on a usage or I/O error.
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import sys

# `test parse_method_ids_20k ... bench:      3324 ns/iter (+/- 31)`
_LINE = re.compile(
    r"^test\s+(?P<name>\S+)\s+\.\.\.\s+bench:\s+(?P<value>\d+)\s+ns/iter"
)
_DATASETS = re.compile(r"window\.BENCHMARK_DATA\s*=\s*(\{.*\})\s*;?\s*$", re.S)


def parse_results(text: str) -> dict[str, float]:
    """libtest-style bench lines -> {bench_name: ns_per_iter}."""
    out: dict[str, float] = {}
    for line in text.splitlines():
        m = _LINE.match(line.strip())
        if m:
            out[m.group("name")] = float(m.group("value"))
    return out


def parse_history(text: str, dataset: str) -> list[dict[str, float]]:
    """`dev/bench/data.js` -> per-commit {bench_name: ns_per_iter}, oldest first.

    The file is a JS assignment wrapping JSON; entries that do not carry the
    requested dataset, or that predate the per-bench shape, are skipped.
    """
    m = _DATASETS.search(text.strip())
    if not m:
        return []
    try:
        data = json.loads(m.group(1))
    except json.JSONDecodeError:
        return []
    runs = (data.get("entries") or {}).get(dataset) or []
    out: list[dict[str, float]] = []
    for entry in runs:
        if not isinstance(entry, dict):
            continue
        benches = entry.get("benches")
        if not isinstance(benches, list):
            continue
        row: dict[str, float] = {}
        for b in benches:
            if isinstance(b, dict) and isinstance(b.get("value"), (int, float)):
                row[str(b.get("name"))] = float(b["value"])
        if row:
            out.append(row)
    return out


def normalized(
    row: dict[str, float], reference: str | None
) -> dict[str, float] | None:
    """{name: ns/iter} -> {name: ns per unit of calibration}.

    Without a reference the row is returned unchanged (raw nanoseconds).
    A row that does not carry the reference bench yields None: it cannot be
    compared with calibrated rows, so it is dropped from the series.
    """
    if not reference:
        return dict(row)
    ref = row.get(reference)
    if ref is None or ref <= 0:
        return None
    return {k: v / ref for k, v in row.items() if k != reference}


def baseline(series: list[dict[str, float]], name: str, window: int) -> float | None:
    """Median of the last `window` stored values for `name`."""
    values = [row[name] for row in series if name in row]
    if not values:
        return None
    return statistics.median(values[-window:])


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Regression gate for bench.yml (runner-class invariant)."
    )
    ap.add_argument("--results", required=True, help="captured `cargo bench` output")
    ap.add_argument("--history", required=True, help="dev/bench/data.js from gh-pages")
    ap.add_argument("--dataset", default="rd-dex parse tables")
    ap.add_argument(
        "--reference",
        default="calib_scan_20k",
        help="bench used as the calibration denominator ('' disables)",
    )
    ap.add_argument("--window", type=int, default=5)
    ap.add_argument("--threshold", type=float, default=0.10)
    ap.add_argument(
        "--min-samples",
        type=int,
        default=3,
        help="calibrated history rows needed before the gate fires",
    )
    args = ap.parse_args(argv)

    if args.window < 1:
        print("bench_gate: --window must be >= 1", file=sys.stderr)
        return 2
    if args.threshold < 0:
        print("bench_gate: --threshold must be >= 0", file=sys.stderr)
        return 2
    if args.min_samples < 1:
        print("bench_gate: --min-samples must be >= 1", file=sys.stderr)
        return 2

    try:
        with open(args.results, encoding="utf-8") as f:
            raw_current = parse_results(f.read())
    except OSError as e:
        print(f"bench_gate: cannot read results: {e}", file=sys.stderr)
        return 2
    if not raw_current:
        print(
            "bench_gate: no `test ... bench:` lines in the results file",
            file=sys.stderr,
        )
        return 2

    reference = args.reference or None
    current = normalized(raw_current, reference)
    if current is None:
        print(
            f"bench_gate: results carry no `{reference}` bench, so they cannot be "
            "normalised; every stored result is normalised the same way, so the "
            "comparison would be meaningless",
            file=sys.stderr,
        )
        return 2

    try:
        with open(args.history, encoding="utf-8") as f:
            raw_history = parse_history(f.read(), args.dataset)
    except OSError:
        raw_history = []

    history = [row for row in map(lambda r: normalized(r, reference), raw_history) if row]

    unit = f"ns per unit of {reference}" if reference else "ns/iter"
    print(
        f"bench_gate: {args.dataset} — baseline = median of last {args.window} "
        f"calibrated results, threshold {args.threshold * 100:.0f}%"
    )
    print(f"  metric: {unit}")

    # Sanity check on the reference itself: if this machine is far outside every
    # class seen so far, say so even though the comparison is still made.
    if reference:
        ref_history = [r[reference] for r in raw_history if reference in r]
        if len(ref_history) >= 3:
            ref_base = statistics.median(ref_history[-args.window :])
            ratio = raw_current[reference] / ref_base
            if ratio > 2.5 or ratio < 0.4:
                print(
                    f"  WARNING: calibration itself is {ratio:.2f}x the median of "
                    f"the last {args.window} runs — this runner is far outside the "
                    "classes seen so far; treat the numbers with care"
                )

    if len(history) < args.min_samples:
        print(
            f"\nbench_gate: only {len(history)} calibrated result(s) in history, "
            f"need {args.min_samples} — seeding, not gating yet (pass)"
        )
        return 0

    failures: list[str] = []
    print(
        f"\n{'benchmark':<28}{'current':>12}{'baseline':>12}{'ratio':>8}"
        f"{'raw ns':>10}  verdict"
    )
    for name in sorted(current):
        cur = current[name]
        base = baseline(history, name, args.window)
        raw = raw_current[name]
        if base is None or base <= 0:
            print(f"{name:<28}{cur:>12.5f}{'—':>12}{'—':>8}{raw:>10.0f}  no baseline (pass)")
            continue
        ratio = cur / base
        bad = ratio > 1.0 + args.threshold
        if bad:
            failures.append(name)
        print(
            f"{name:<28}{cur:>12.5f}{base:>12.5f}{ratio:>8.2f}{raw:>10.0f}  "
            f"{'REGRESSION' if bad else 'ok'}"
        )

    if failures:
        print(
            f"\nbench_gate: {len(failures)} benchmark(s) regressed more than "
            f"{args.threshold * 100:.0f}% over the median baseline: "
            f"{', '.join(failures)}"
        )
        return 1
    print("\nbench_gate: all benchmarks within threshold")
    return 0


if __name__ == "__main__":
    sys.exit(main())
