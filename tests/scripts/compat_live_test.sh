#!/usr/bin/env bash
# Self-tests for scripts/compat_live.py (página "compatibilidade ao vivo").
# Hermetic: fixtures em mktemp, sem rede, sem mdbook, sem cargo.
#
#   bash tests/scripts/compat_live_test.sh
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
TOOL="$ROOT/scripts/compat_live.py"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fails=0
ok() { echo "ok   - $1"; }
ko() { echo "FAIL - $1" >&2; fails=$((fails + 1)); }
run() {
  local want="$1" name="$2"
  shift 2
  local got=0
  python3 "$TOOL" "$@" >"$WORK/out" 2>&1 || got=$?
  if [ "$got" -eq "$want" ]; then ok "$name"; else ko "$name (exit $got, want $want)"; sed 's/^/     | /' "$WORK/out" >&2; fi
}
has() {
  local needle="$1" name="$2" file="$3"
  if grep -qF -- "$needle" "$file"; then ok "$name"; else ko "$name (não achei: $needle)"; sed 's/^/     | /' "$file" >&2; fi
}
hasnt() {
  local needle="$1" name="$2" file="$3"
  if grep -qF -- "$needle" "$file"; then ko "$name (achei o que não devia: $needle)"; else ok "$name"; fi
}

PAGE="$WORK/compat-live.md"
cat >"$PAGE" <<'MD'
# Compatibilidade ao vivo

<!-- compat-live:begin -->
placeholder
<!-- compat-live:end -->

rodapé
MD

# ── 1. sem dados: página continua válida e diz que não há dados ────────────
run 0 "sem relatórios: roda" --reports "$WORK/none" --page "$PAGE" --badge "$WORK/badge.svg"
has "Sem dados recentes" "sem dados: bloco honesto" "$PAGE"
hasnt "placeholder" "sem dados: marcador substituído" "$PAGE"
has "#9e9e9e" "sem dados: badge cinza" "$WORK/badge.svg"
has "rodapé" "sem dados: resto da página intacto" "$PAGE"

# ── 2. fixtures: uma suíte verde, uma com falha ───────────────────────────
REP="$WORK/reports"
mkdir -p "$REP/junit" "$REP/inspect"
cat >"$REP/junit/m8-compat.xml" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="rustydroid" tests="12" failures="0" errors="0" time="1.500">
  <testsuite name="m8-compat" tests="12" failures="0" errors="0" skipped="1" time="1.500">
    <testcase classname="m8-compat" name="a" time="0.100"/>
  </testsuite>
</testsuites>
XML
cat >"$REP/junit/contract-no-app.xml" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="rustydroid" tests="5" failures="2" errors="0" time="0.500">
  <testsuite name="contract-no-app" tests="5" failures="2" errors="0" skipped="0" time="0.500">
    <testcase classname="contract-no-app" name="b" time="0.100"/>
  </testsuite>
</testsuites>
XML
echo '{"app":"x"}' >"$REP/inspect/org.schabi.newpipe.json"
echo '{"app":"y"}' >"$REP/inspect/org.tasks.json"
cat >"$WORK/run.json" <<'JSON'
{"conclusion":"success","head_sha":"0288764abcdef","created_at":"2026-11-10T04:43:12Z","run_number":42,"html_url":"https://github.com/nadabgfccvd/RustyDroid/actions/runs/1"}
JSON

run 0 "com falha: roda" --reports "$REP" --run-json "$WORK/run.json" --page "$PAGE" --badge "$WORK/badge.svg"
has "❌" "com falha: veredito vermelho" "$PAGE"
has "**total** | **17** | **2**" "com falha: total de testes e falhas" "$PAGE"
has "contract-no-app" "com falha: suíte listada" "$PAGE"
has "0288764" "com falha: sha curto da run" "$PAGE"
has "2** APK(s) inspecionado(s)" "com falha: contagem de inspect" "$PAGE"
has "#c62828" "com falha: badge vermelho" "$WORK/badge.svg"
has "17 testes" "com falha: badge com total" "$WORK/badge.svg"

# ── 3. tudo verde: veredito ✅ e badge verde ──────────────────────────────
rm "$REP/junit/contract-no-app.xml"
sed -i 's/failures="2"/failures="0"/' "$REP/junit/m8-compat.xml"
run 0 "tudo verde: roda" --reports "$REP" --run-json "$WORK/run.json" --page "$PAGE" --badge "$WORK/badge.svg"
has "✅" "tudo verde: veredito verde" "$PAGE"
has "#2e7d32" "tudo verde: badge verde" "$WORK/badge.svg"

# ── 4. determinismo: duas execuções, mesmos bytes ─────────────────────────
cp "$PAGE" "$WORK/page.1"
cp "$WORK/badge.svg" "$WORK/badge.1"
python3 "$TOOL" --reports "$REP" --run-json "$WORK/run.json" --page "$PAGE" --badge "$WORK/badge.svg" --quiet
if cmp -s "$PAGE" "$WORK/page.1" && cmp -s "$WORK/badge.svg" "$WORK/badge.1"; then
  ok "determinístico (página e badge idênticos byte a byte)"
else
  ko "determinístico (saída mudou entre execuções)"
fi

# ── 5. XML corrompido não derruba a página ────────────────────────────────
echo "<not-json" >"$REP/junit/broken.xml"
run 0 "xml corrompido: roda" --reports "$REP" --page "$PAGE" --badge "$WORK/badge.svg"
hasnt "broken" "xml corrompido: suíte ilegível ignorada" "$PAGE"
rm "$REP/junit/broken.xml"

# ── 6. página sem marcadores é erro, não escrita silenciosa ───────────────
BAD="$WORK/bad.md"
printf '# nada aqui\n' >"$BAD"
run 1 "sem marcadores: falha com exit 1" --reports "$REP" --page "$BAD" --badge "$WORK/b2.svg"
if [ "$(cat "$BAD")" = "# nada aqui" ]; then ok "sem marcadores: página intacta"; else ko "sem marcadores: página foi mexida"; fi

# ── 7. --check não escreve ────────────────────────────────────────────────
cp "$PAGE" "$WORK/page.before"
run 0 "--check roda" --check --reports "$REP" --page "$PAGE"
if cmp -s "$PAGE" "$WORK/page.before"; then ok "--check não escreve"; else ko "--check escreveu na página"; fi

echo
if [ "$fails" -eq 0 ]; then
  echo "compat_live_test: todos os testes passaram"
else
  echo "compat_live_test: $fails falha(s)" >&2
  exit 1
fi
