#!/usr/bin/env bash
# M7 golden perf gate (PRF-06, Apêndice A2): a pinned, launchable real APK
# (org.primftpd 8.2, versionCode 71) must meet the moto-e5 budgets via
# `rd perf`. This is the golden gate the ROADMAP M7 DoD asks for — the
# trivial-app gate lives in crates/rd-cli/tests/perf_cli.rs.
# APK is cached under golden-apks/cache/ (never committed, GH-05).
# Requires: curl, sha256sum, python3. Output: target/m7-golden-perf.json;
# exit 0 = budgets ok · 1 = budget failed or PERF_FAILED.
set -euo pipefail
cd "$(dirname "$0")/.."
CACHE=golden-apks/cache; mkdir -p "$CACHE" target
cargo build --release -q -p rd-cli

fetch() { # file sha256 (F-Droid moves superseded versions from repo/ to archive/)
  local f="$CACHE/$1"
  if [ ! -f "$f" ] || ! echo "$2  $f" | sha256sum -c --quiet - 2>/dev/null; then
    curl -fsSL -o "$f" "https://f-droid.org/repo/$1" || curl -fsSL -o "$f" "https://f-droid.org/archive/$1"
  fi
  echo "$2  $f" | sha256sum -c --quiet -
}
fetch org.primftpd_71.apk d36566e774e2d0446dcbb41438d22e62d198688aac2711ba7afaa6ea2fdd577d

# O relatório JSON sai no stdout mesmo quando o orçamento falha (exit 1);
# guardamos o rc, imprimimos o resumo a partir do JSON e propagamos a falha.
set +e
./target/release/rd perf "$CACHE/org.primftpd_71.apk" \
  --device moto-e5 --runs 3 --json > target/m7-golden-perf.json
rc=$?
set -e

python3 - target/m7-golden-perf.json "$rc" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
rc = int(sys.argv[2])
print(f"org.primftpd 8.2 vs moto-e5 budgets: {'PASS' if d['pass'] else 'FAIL'}")
for b in d["budgets"]:
    actual = "não medido" if b["actual"] is None else f"{b['actual']} {b['unit']}"
    mark = "ok" if b["pass"] else "FALHOU"
    print(f"  {b['name']:30s} {actual:>14} / {b['limit']} {b['unit']}  {mark}")
cold = [round(r["emulated_ms"]) for r in d["cold"]]
warm = [round(r["emulated_ms"]) for r in d["warm"]]
print(f"  cold emulado (ms): {cold}")
print(f"  warm emulado (ms): {warm}")
print("resultado do gate:", "PASSOU" if rc == 0 else "FALHOU")
PY
exit "$rc"
