#!/usr/bin/env bash
# Self-tests for scripts/releases_live.py (página de releases do site).
# Hermetic: fixtures em mktemp, sem rede, sem cargo.
#
#   bash tests/scripts/releases_live_test.sh
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
TOOL="$ROOT/scripts/releases_live.py"
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
  if grep -qF -- "$needle" "$file"; then ok "$name"; else ko "$name (não achei: $needle)"; fi
}
hasnt() {
  local needle="$1" name="$2" file="$3"
  if grep -qF -- "$needle" "$file"; then ko "$name (achei o que não devia: $needle)"; else ok "$name"; fi
}

PAGE="$WORK/releases.md"
cat >"$PAGE" <<'MD'
# Releases

<!-- releases-live:begin -->
placeholder
<!-- releases-live:end -->

rodapé
MD

# ── 1. resposta válida e vazia: bloco "nenhuma release" ───────────────────
echo '[]' >"$WORK/empty.json"
run 0 "lista vazia: roda" --api-json "$WORK/empty.json" --page "$PAGE"
has "Nenhuma release publicada ainda" "lista vazia: bloco honesto" "$PAGE"
hasnt "placeholder" "lista vazia: marcador substituído" "$PAGE"
has "rodapé" "lista vazia: resto da página intacto" "$PAGE"

# ── 2. duas releases, uma com assets ─────────────────────────────────────
cat >"$WORK/two.json" <<'JSON'
[
  {"tag_name":"v1.0.0","published_at":"2026-11-20T10:00:00Z","html_url":"https://github.com/x/y/releases/tag/v1.0.0",
   "assets":[
     {"name":"SHA256SUMS.txt","size":128,"browser_download_url":"https://example.com/SHA256SUMS.txt"},
     {"name":"rustydroid-v1.0.0-linux-x86_64.tar.gz","size":15728640,"browser_download_url":"https://example.com/a.tar.gz"}
   ]},
  {"tag_name":"v0.9.0","published_at":"2026-10-19T07:23:36Z","html_url":"https://github.com/x/y/releases/tag/v0.9.0",
   "assets":[]}
]
JSON
run 0 "duas releases: roda" --api-json "$WORK/two.json" --page "$PAGE"
has "**2 release(s)**" "duas releases: contagem" "$PAGE"
has "**2** artefato(s)" "duas releases: total de artefatos" "$PAGE"
has "| [\`v1.0.0\`](https://github.com/x/y/releases/tag/v1.0.0) | 2026-11-20 | 2 | 15.00 MiB |" \
  "duas releases: linha com data, contagem e tamanho" "$PAGE"
has "| [\`v0.9.0\`](https://github.com/x/y/releases/tag/v0.9.0) | 2026-10-19 | 0 | — |" \
  "duas releases: release sem asset mostra travessão" "$PAGE"
has "rustydroid-v1.0.0-linux-x86_64.tar.gz" "duas releases: nome do tarball listado" "$PAGE"
has "15.00 MiB" "duas releases: tamanho humanizado" "$PAGE"
has "attestation verify" "duas releases: instrução de verificação de procedência" "$PAGE"
# A release sem assets não abre um bloco de detalhes vazio.
[ "$(grep -c '<details>' "$PAGE")" -eq 1 ] \
  && ok "duas releases: um único <details> (só a que tem assets)" \
  || ko "duas releases: <details> a mais"

# ── 3. determinismo ──────────────────────────────────────────────────────
cp "$PAGE" "$WORK/page.1"
python3 "$TOOL" --api-json "$WORK/two.json" --page "$PAGE" --quiet
cmp -s "$PAGE" "$WORK/page.1" \
  && ok "determinístico (byte a byte)" \
  || ko "determinístico (saída mudou)"

# ── 4. falha da API não apaga o histórico publicado ──────────────────────
# Esta é a diferença de política em relação à página de compatibilidade: uma
# lista de releases não envelhece, então sumir com ela seria pior que ficar
# um dia desatualizada.
cp "$PAGE" "$WORK/page.1"
run 0 "sem arquivo da API: roda" --api-json "$WORK/nao-existe.json" --page "$PAGE"
cmp -s "$PAGE" "$WORK/page.1" \
  && ok "sem arquivo da API: página intacta" \
  || ko "sem arquivo da API: página foi reescrita"

echo '{"message":"API rate limit exceeded"}' >"$WORK/err.json"
run 0 "erro da API (objeto, não lista): roda" --api-json "$WORK/err.json" --page "$PAGE"
cmp -s "$PAGE" "$WORK/page.1" \
  && ok "erro da API: página intacta" \
  || ko "erro da API: página foi reescrita"

printf 'not json at all' >"$WORK/broken.json"
run 0 "JSON corrompido: roda" --api-json "$WORK/broken.json" --page "$PAGE"
cmp -s "$PAGE" "$WORK/page.1" \
  && ok "JSON corrompido: página intacta" \
  || ko "JSON corrompido: página foi reescrita"

# ── 5. entradas sem tag_name são descartadas, não derrubam o render ──────
cat >"$WORK/partial.json" <<'JSON'
[{"name":"sem tag, deve ser ignorada"},{"tag_name":"v0.1.0","assets":[]}]
JSON
run 0 "entrada sem tag: roda" --api-json "$WORK/partial.json" --page "$PAGE"
has "**1 release(s)**" "entrada sem tag: descartada" "$PAGE"
hasnt "v0.1.0 " "entrada sem tag: só a válida aparece" "$PAGE" || true
has "v0.1.0" "entrada sem tag: a válida continua" "$PAGE"

# ── 6. página sem marcadores é erro, sem escrita silenciosa ──────────────
BAD="$WORK/bad.md"
printf '# nada\n' >"$BAD"
run 2 "sem marcadores: exit 2" --api-json "$WORK/two.json" --page "$BAD"
[ "$(cat "$BAD")" = "# nada" ] \
  && ok "sem marcadores: página intacta" \
  || ko "sem marcadores: página foi mexida"

# ── 7. --check não escreve ───────────────────────────────────────────────
cp "$PAGE" "$WORK/page.1"
run 0 "--check roda" --check --api-json "$WORK/two.json" --page "$PAGE"
cmp -s "$PAGE" "$WORK/page.1" \
  && ok "--check não escreve" \
  || ko "--check escreveu na página"

echo
if [ "$fails" -eq 0 ]; then
  echo "releases_live_test: todos os testes passaram"
else
  echo "releases_live_test: $fails falha(s)" >&2
  exit 1
fi
