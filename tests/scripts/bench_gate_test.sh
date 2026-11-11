#!/usr/bin/env bash
# Self-tests for the bench regression gate (scripts/bench_gate.py, Issue #629).
# Hermetic: only writes fixtures under a mktemp dir. No network, no cargo.
#
#   bash tests/scripts/bench_gate_test.sh
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
GATE="$ROOT/scripts/bench_gate.py"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fails=0
ok() { echo "ok   - $1"; }
ko() { echo "FAIL - $1" >&2; fails=$((fails + 1)); }

# run_gate <expect_exit> <name> <args...>   (stdout+stderr captured in $WORK/out)
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

# assert_out <pattern> <name>
assert_out() {
  if grep -qF -- "$1" "$WORK/out"; then
    ok "$2"
  else
    ko "$2 (missing: $1)"
    sed 's/^/     | /' "$WORK/out" >&2
  fi
}

# ── fixtures ───────────────────────────────────────────────────────────────
# A calibrated history whose normalised ratio for both benches is ~0.063.
cat >"$WORK/history.js" <<'JS'
window.BENCHMARK_DATA = {"lastUpdate":0,"repoUrl":"x","entries":{"rd-dex parse tables":[
 {"commit":{"id":"a1"},"benches":[
   {"name":"calib_scan_20k","value":60000},
   {"name":"parse_method_ids_20k","value":3832},
   {"name":"parse_field_ids_20k","value":3786}]},
 {"commit":{"id":"a2"},"benches":[
   {"name":"calib_scan_20k","value":61000},
   {"name":"parse_method_ids_20k","value":3929},
   {"name":"parse_field_ids_20k","value":3831}]},
 {"commit":{"id":"a3"},"benches":[
   {"name":"calib_scan_20k","value":59000},
   {"name":"parse_method_ids_20k","value":3747},
   {"name":"parse_field_ids_20k","value":3717}]},
 {"commit":{"id":"a4"},"benches":[
   {"name":"calib_scan_20k","value":59800},
   {"name":"parse_method_ids_20k","value":3821},
   {"name":"parse_field_ids_20k","value":3767}]},
 {"commit":{"id":"a5"},"benches":[
   {"name":"calib_scan_20k","value":60500},
   {"name":"parse_method_ids_20k","value":3876},
   {"name":"parse_field_ids_20k","value":3808}]}
]}};
JS

# Same code on the machine the history was recorded on.
cat >"$WORK/fast.txt" <<'TXT'
test calib_scan_20k ... bench:        59178 ns/iter (+/- 3849)
test parse_method_ids_20k ... bench:         3779 ns/iter (+/- 50)
test parse_field_ids_20k ... bench:          3816 ns/iter (+/- 129)
TXT

# The exact scenario that broke CI: identical code, a runner ~5x slower.
# Every raw number is ~5x the history's, so only normalisation can pass this.
cat >"$WORK/fast_but_slow_runner.txt" <<'TXT'
test calib_scan_20k ... bench:       295890 ns/iter (+/- 9000)
test parse_method_ids_20k ... bench:         18895 ns/iter (+/- 400)
test parse_field_ids_20k ... bench:          19080 ns/iter (+/- 410)
TXT

# A real +15% regression on the same slow runner: must still be caught.
cat >"$WORK/regressed_slow_runner.txt" <<'TXT'
test calib_scan_20k ... bench:       295890 ns/iter (+/- 9000)
test parse_method_ids_20k ... bench:         21730 ns/iter (+/- 400)
test parse_field_ids_20k ... bench:          21940 ns/iter (+/- 410)
TXT

# A real +15% regression on the fast runner.
cat >"$WORK/regressed.txt" <<'TXT'
test calib_scan_20k ... bench:        59178 ns/iter (+/- 3849)
test parse_method_ids_20k ... bench:         4346 ns/iter (+/- 50)
test parse_field_ids_20k ... bench:          4388 ns/iter (+/- 129)
TXT

# Results from a bench binary that predates the calibration workload.
cat >"$WORK/no_calib.txt" <<'TXT'
test parse_method_ids_20k ... bench:         3779 ns/iter (+/- 50)
test parse_field_ids_20k ... bench:          3816 ns/iter (+/- 129)
TXT

# History from before the calibration workload landed (188 such rows upstream).
cat >"$WORK/history_raw.js" <<'JS'
window.BENCHMARK_DATA = {"entries":{"rd-dex parse tables":[
 {"commit":{"id":"b1"},"benches":[
   {"name":"parse_method_ids_20k","value":3324},
   {"name":"parse_field_ids_20k","value":3340}]}
]}};
JS

# ── cases ──────────────────────────────────────────────────────────────────
run_gate 0 "same code on the recording runner passes" \
  --results "$WORK/fast.txt" --history "$WORK/history.js"
assert_out "all benchmarks within threshold" "reports a clean pass"

run_gate 0 "same code on a 5x slower runner passes (#629)" \
  --results "$WORK/fast_but_slow_runner.txt" --history "$WORK/history.js"
assert_out "within threshold" "5x slower runner is not flagged"
assert_out "WARNING: calibration itself is" "5x slower runner still warns"

run_gate 1 "a +15% regression on the recording runner fails" \
  --results "$WORK/regressed.txt" --history "$WORK/history.js"
assert_out "REGRESSION" "regression is named"

run_gate 1 "a +15% regression on a 5x slower runner fails" \
  --results "$WORK/regressed_slow_runner.txt" --history "$WORK/history.js"
assert_out "regressed more than 10%" "regression is named on a foreign runner"

run_gate 0 "missing history seeds instead of failing" \
  --results "$WORK/fast.txt" --history "$WORK/does-not-exist.js"
assert_out "seeding" "missing history reports seeding"

run_gate 0 "history predating the calibration bench seeds" \
  --results "$WORK/fast.txt" --history "$WORK/history_raw.js"
assert_out "seeding" "uncalibrated history reports seeding"

run_gate 2 "results without the calibration bench are a usage error" \
  --results "$WORK/no_calib.txt" --history "$WORK/history.js"

: >"$WORK/empty.txt"
run_gate 2 "an empty results file is a usage error" \
  --results "$WORK/empty.txt" --history "$WORK/history.js"

# --reference '' opts out of normalisation: raw ns against raw ns.
run_gate 0 "raw mode compares raw values" \
  --results "$WORK/no_calib.txt" --history "$WORK/history_raw.js" --reference ''

run_gate 1 "raw mode still catches a regression" \
  --results "$WORK/regressed.txt" --history "$WORK/history.js" --reference ''
assert_out "REGRESSION" "raw mode names the regression"

run_gate 0 "--window 1 uses only the last sample" \
  --results "$WORK/fast.txt" --history "$WORK/history.js" --window 1

run_gate 1 "a tighter threshold surfaces small drifts" \
  --results "$WORK/fast.txt" --history "$WORK/history.js" --threshold 0.01

run_gate 2 "--window 0 is rejected" \
  --results "$WORK/fast.txt" --history "$WORK/history.js" --window 0

# A truncated / hand-edited data.js must not crash the gate.
printf 'window.BENCHMARK_DATA = {this is not json;' >"$WORK/broken.js"
run_gate 0 "malformed history does not crash" \
  --results "$WORK/fast.txt" --history "$WORK/broken.js"

echo
if [ "$fails" -ne 0 ]; then
  echo "$fails check(s) failed" >&2
  exit 1
fi
echo "all checks passed"
