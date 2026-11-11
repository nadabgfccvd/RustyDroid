#!/usr/bin/env bash
# Self-tests for scripts/book_links.py (docs site rewrite).
# Hermetic: a fake site under mktemp. No mdbook, no network.
#
#   bash tests/scripts/book_links_test.sh
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
TOOL="$ROOT/scripts/book_links.py"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

fails=0
ok() { echo "ok   - $1"; }
ko() { echo "FAIL - $1" >&2; fails=$((fails + 1)); }
run() { local want="$1" name="$2"; shift 2; local got=0; python3 "$TOOL" "$@" >"$WORK/out" 2>&1 || got=$?; if [ "$got" -eq "$want" ]; then ok "$name"; else ko "$name (exit $got, want $want)"; sed 's/^/     | /' "$WORK/out" >&2; fi; }

# ── fixture: a miniature site ──────────────────────────────────────────────
SITE="$WORK/site"
mkdir -p "$SITE/adr" "$SITE/css"
echo 'x' >"$SITE/favicon-abc.svg"
echo 'x' >"$SITE/css/variables-abc.css"
cat >"$SITE/adr/page.html" <<'HTML'
<a href="../css/variables-abc.css">style (valid asset, must be kept)</a>
<a href="../favicon-abc.svg">icon (valid asset, must be kept)</a>
<a href="../README.md">escapes the book (must be rewritten)</a>
<a href="../../docs/ROADMAP.md">escapes twice (must be rewritten)</a>
HTML
cp "$SITE/adr/page.html" "$WORK/original.html"

run 0 "assets are kept, escapes are rewritten" --site "$SITE"

grep -q 'href="../css/variables-abc.css"' "$SITE/adr/page.html" \
  && ok "mdBook css asset untouched" \
  || ko "mdBook css asset was rewritten"
grep -q 'href="../favicon-abc.svg"' "$SITE/adr/page.html" \
  && ok "mdBook favicon untouched" \
  || ko "mdBook favicon was rewritten"
grep -q 'href="https://github.com/nadabgfccvd/RustyDroid/blob/main/README.md"' "$SITE/adr/page.html" \
  && ok "escaping link points at the GitHub blob" \
  || ko "escaping link was not rewritten"
grep -q 'blob/main/docs/ROADMAP.md' "$SITE/adr/page.html" \
  && ok "double escape is resolved from the repo root" \
  || ko "double escape resolved wrongly"
grep -q 'README.md"' "$SITE/adr/page.html" && ! grep -q 'href="../README.md"' "$SITE/adr/page.html" \
  && ok "no leftover ../README.md" \
  || ko "leftover ../README.md"

# Idempotent: a second run must change nothing.
before="$(cat "$SITE/adr/page.html")"
run 0 "second run is a no-op" --site "$SITE"
if [ "$before" = "$(cat "$SITE/adr/page.html")" ]; then
  ok "idempotent"
else
  ko "second run changed the page"
fi

# --check must not write.
cp "$WORK/original.html" "$SITE/adr/page.html"
run 0 "--check reports without writing" --site "$SITE" --check
if [ "$(cat "$WORK/original.html")" = "$(cat "$SITE/adr/page.html")" ]; then
  ok "--check left the file alone"
else
  ko "--check wrote to disk"
fi

# A page with only valid links: nothing to report, still exit 0.
cat >"$SITE/adr/clean.html" <<'HTML'
<a href="../css/variables-abc.css">only assets</a>
HTML
run 0 "a clean page passes" --site "$SITE"

# A missing site directory is a usage error, not a silent pass.
run 2 "missing site directory is a usage error" --site "$WORK/nope"

# The counter must not count preserved matches (the bug this test exists for:
# re.subn counts matches, so 918 untouched assets were reported as rewrites).
cp "$WORK/original.html" "$SITE/adr/page.html"
python3 "$TOOL" --site "$SITE" >"$WORK/out2" 2>&1
if grep -q "reescreveu 2 link" "$WORK/out2"; then
  ok "reports 2 rewrites, not 4 matches"
else
  ko "wrong rewrite count: $(cat "$WORK/out2")"
fi

# ── mdBook renders `X/README.md` as `X/index.html` ────────────────────────
# So a (repo-correct) link to `./archive/README.md` becomes `archive/README.html`
# in the built site, which 404s. It must be rewritten to the page that exists.
# Fixture isolado para não mexer na contagem de reescritas acima.
SITE2="$WORK/site2"
mkdir -p "$SITE2/archive"
echo '<h1>arquivo</h1>' >"$SITE2/archive/index.html"
cat >"$SITE2/page.html" <<'HTML'
<a href="./archive/README.html">arquivo (README vira index)</a>
<a href="https://example.com/README.html">link externo (não tocar)</a>
HTML
run 0 "README.html -> index.html quando o index existe" --site "$SITE2"
grep -q 'href="./archive/index.html"' "$SITE2/page.html" \
  && ok "README.html reescrito para index.html" \
  || ko "README.html não foi reescrito"
grep -q 'href="https://example.com/README.html"' "$SITE2/page.html" \
  && ok "link externo para README.html preservado" \
  || ko "link externo foi mexido"
before2="$(cat "$SITE2/page.html")"
run 0 "segunda passada do README é no-op" --site "$SITE2"
[ "$before2" = "$(cat "$SITE2/page.html")" ] \
  && ok "README.html: idempotente" \
  || ko "README.html: segunda passada mexeu no arquivo"

# Sem `index.html` substituto o link é alvo inexistente: o build tem de falhar,
# não publicar um 404 silencioso.
cat >"$SITE2/broken.html" <<'HTML'
<a href="./nada/README.html">destino que não existe</a>
HTML
run 1 "README.html sem index é falha" --site "$SITE2"
grep -q 'href="./nada/README.html"' "$SITE2/broken.html" \
  && ok "alvo inexistente continua intacto (falha, não correção inventada)" \
  || ko "alvo inexistente foi reescrito para algo arbitrário"
rm "$SITE2/broken.html"

echo
if [ "$fails" -ne 0 ]; then echo "$fails check(s) failed" >&2; exit 1; fi
echo "all checks passed"
