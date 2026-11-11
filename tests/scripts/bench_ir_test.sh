#!/usr/bin/env bash
# Self-tests for the instruction-count regression gate (scripts/bench_ir.py,
# Issue #629 / ADR 038). Hermetic: fixtures under mktemp, no valgrind, no cargo,
# no network. The counter itself is covered by the workflow that runs it; this
# covers the comparison logic, which is where a gate silently rots.
#
#   bash tests/scripts/bench_ir_test.sh
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
GATE="$ROOT/scripts/bench_ir.py"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fails=0
ok() { echo "ok   - $1"; }
ko() { echo "FAIL - $1" >&2; fails=$((fails + 1)); }

# run_gate <expect_exit> <name> <args...>
run_gate() {
  local want="$1" name="$2"
  shift 2
  local got=0
  python3 "$GATE" "$@" >"$WORK/out" 2>&1 || got=$?
  if [ "$got" -eq "$want" ]; then
    ok "$name (exit $got)"
  else
    ko "$name (exit $got, want $want)"
    sed 's/^/     | /' "$WORK/out" >&2
  fi
}

assert_out() {
  if grep -qF -- "$1" "$WORK/out"; then
    ok "$2"
  else
    ko "$2 (missing: $1)"
    sed 's/^/     | /' "$WORK/out" >&2
  fi
}

# ── fixtures ───────────────────────────────────────────────────────────────
# Measured on the real bench: noop 878 280, parse_* 923 8xx, so the work delta
# is ~45 5xx instructions. The baseline below pins 45 500.
cat >"$WORK/baseline.json" <<'JSON'
{
  "parse_field_ids_20k": 45570,
  "parse_method_ids_20k": 45540
}
JSON

NOOP=878280
cat >"$WORK/clean.txt" <<EOF
ir noop $NOOP
ir parse_method_ids_20k $((NOOP + 45540))
ir parse_field_ids_20k $((NOOP + 45570))
EOF

# +9.9%: under the 10% rule, must pass.
cat >"$WORK/within.txt" <<EOF
ir noop $NOOP
ir parse_method_ids_20k $((NOOP + 45540 * 1099 / 1000))
ir parse_field_ids_20k $((NOOP + 45570))
EOF

# +11%: over the rule, must fail.
cat >"$WORK/regressed.txt" <<EOF
ir noop $NOOP
ir parse_method_ids_20k $((NOOP + 45540 * 111 / 100))
ir parse_field_ids_20k $((NOOP + 45570))
EOF

# Fewer instructions is an improvement, never a failure.
cat >"$WORK/faster.txt" <<EOF
ir noop $NOOP
ir parse_method_ids_20k $((NOOP + 30000))
ir parse_field_ids_20k $((NOOP + 45570))
EOF

# ── cases ──────────────────────────────────────────────────────────────────
run_gate 0 "identical counts pass" --results "$WORK/clean.txt" --baseline "$WORK/baseline.json"
assert_out "dentro do limiar" "reports a clean pass"

run_gate 0 "+9.9% still passes" --results "$WORK/within.txt" --baseline "$WORK/baseline.json"

run_gate 1 "+11% fails" --results "$WORK/regressed.txt" --baseline "$WORK/baseline.json"
assert_out "REGRESSÃO" "regression is named"
assert_out "atualize o baseline" "failure tells the author how to update the baseline"

run_gate 0 "fewer instructions is not a regression" \
  --results "$WORK/faster.txt" --baseline "$WORK/baseline.json"

# The noop run is the process overhead and must never be compared against a
# baseline — only subtracted. If it leaked into the comparison, every run would
# report a 19x "regression".
run_gate 0 "noop is subtracted, not compared" \
  --results "$WORK/clean.txt" --baseline "$WORK/baseline.json"
if grep -q "noop" "$WORK/out"; then
  ko "noop leaked into the comparison table"
else
  ok "noop is absent from the comparison table"
fi

# A different noop value must not change the verdict: it is overhead, and the
# gate compares the delta.
cat >"$WORK/different_noop.txt" <<EOF
ir noop 999999
ir parse_method_ids_20k $((999999 + 45540))
ir parse_field_ids_20k $((999999 + 45570))
EOF
run_gate 0 "a different noop does not move the verdict" \
  --results "$WORK/different_noop.txt" --baseline "$WORK/baseline.json"

# No baseline file: seed instead of failing a brand-new gate.
run_gate 0 "missing baseline seeds" --results "$WORK/clean.txt" --baseline "$WORK/nope.json"
assert_out "no baseline at" "missing baseline is reported"

# --write creates the baseline from work counts only (noop excluded).
run_gate 0 "--write creates the baseline" \
  --results "$WORK/clean.txt" --baseline "$WORK/fresh.json" --write
if grep -q '"noop"' "$WORK/fresh.json"; then
  ko "--write persisted the noop row"
else
  ok "--write excludes the noop row"
fi
if grep -q '"parse_method_ids_20k": 45540' "$WORK/fresh.json"; then
  ok "--write stores the work delta"
else
  ko "--write stored the wrong value: $(cat "$WORK/fresh.json")"
fi

# A bench present in the results but absent from the baseline is announced, not
# silently ignored — a new bench with no baseline is a gate hole.
cat >"$WORK/new_bench.txt" <<EOF
ir noop $NOOP
ir parse_method_ids_20k $((NOOP + 45540))
ir parse_string_ids_20k $((NOOP + 1234))
EOF
run_gate 0 "an unknown bench does not fail the run" \
  --results "$WORK/new_bench.txt" --baseline "$WORK/baseline.json"
assert_out "sem baseline" "an unknown bench is announced"

# Usage errors.
printf 'garbage\n' >"$WORK/malformed.txt"
run_gate 2 "malformed results are a usage error" \
  --results "$WORK/malformed.txt" --baseline "$WORK/baseline.json"

: >"$WORK/empty.txt"
run_gate 2 "empty results are a usage error" \
  --results "$WORK/empty.txt" --baseline "$WORK/baseline.json"

run_gate 2 "missing results file is a usage error" \
  --results "$WORK/ghost.txt" --baseline "$WORK/baseline.json"

# Only a noop row means there is no work to compare.
printf 'ir noop 100\n' >"$WORK/only_noop.txt"
run_gate 2 "a noop-only run is a usage error" \
  --results "$WORK/only_noop.txt" --baseline "$WORK/baseline.json"

run_gate 2 "a negative threshold is rejected" \
  --results "$WORK/clean.txt" --baseline "$WORK/baseline.json" --threshold -1

# The shipped baseline must stay in sync with the bench targets: every target the
# workflow counts needs a baseline entry, or the gate has a hole.
shipped="$ROOT/crates/rd-dex/benches/ir-baseline.json"
bench="$ROOT/crates/rd-dex/benches/parse_tables.rs"
for target in parse_method_ids_20k parse_field_ids_20k; do
  if grep -q "\"$target\"" "$shipped"; then
    ok "shipped baseline covers $target"
  else
    ko "shipped baseline is missing $target"
  fi
  if grep -q "&\?\"$target\"" "$bench"; then
    ok "bench defines $target"
  else
    ko "bench does not define $target"
  fi
done
if grep -q '"noop"' "$shipped"; then
  ko "shipped baseline must not contain the noop row"
else
  ok "shipped baseline has no noop row"
fi

echo
if [ "$fails" -ne 0 ]; then
  echo "$fails check(s) failed" >&2
  exit 1
fi
echo "all checks passed"
