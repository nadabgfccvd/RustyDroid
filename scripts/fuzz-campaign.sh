#!/usr/bin/env bash
#
# scripts/fuzz-campaign.sh — Campanha de fuzz 24h (DoD do M1) — ver docs/FUZZING.md
#
# Política (docs/FUZZING.md): os 5 targets (zip, axml, arsc, apk_full, dex)
# rodam em PARALELO por N segundos (padrão 86400 = 24 h), em modo -fork com
# -ignore_crashes=1 — o fuzzer CONTINUA após crash/oom/timeout e arquiva cada
# achado em crates/*/fuzz/artifacts/.
#
#   DoD atingido  = ao final, NENHUM arquivo `crash-*` nos artifacts.
#   DoD violado   = qualquer `crash-*` é bug P0 (Lei nº 1) → minimizar + issue.
#                   (oom-*/timeout-* são AVISOS — investigar depois; não violam
#                    o DoD escrito, que é "sem crash".)
#
# Uso:
#   ./scripts/fuzz-campaign.sh             # 24 h (86400 s), seeds automáticos
#   ./scripts/fuzz-campaign.sh 600         # smoke de 10 min
#   ./scripts/fuzz-campaign.sh 28800 4     # 8 h, 4 workers por target
#
# Requisitos: rustup + toolchain nightly + cargo-fuzz@0.13.2 (versão do CI) +
# unzip + curl (fetch do APK golden). Corpus persiste em crates/*/fuzz/corpus/
# (gitignored) — rodar de novo CONTINUA a campanha a partir do corpus existente.
#
# DISCO: o corpus cresce bastante nas primeiras horas (unidades novas têm até
# o tamanho de max_len). Cheque de vez em quando:
#     du -sh crates/*/fuzz/corpus
# Se apertar o disco, PARE, minimize e retome (o merge NÃO perde cobertura):
#     cd crates/rd-apk && cargo fuzz merge corpus corpus.tmp && rm -rf corpus.tmp
#     cd crates/rd-dex && cargo fuzz merge corpus corpus.tmp && rm -rf corpus.tmp

set -euo pipefail

SECS="${1:-86400}"                 # tempo total por target (s)
WORKERS="${2:-0}"                  # 0 = auto: dividir os cores entre 5 targets
REPO="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
APK="${REPO}/golden-apks/org.fdroid.fdroid.apk"
LOGDIR="${REPO}/fuzz-logs/$(date +%Y%m%d-%H%M%S)"

fail=0                             # targets que falharam (build/execução)

if ! command -v cargo >/dev/null; then
    echo "ERRO: cargo não está no PATH — rode: source \"\$HOME/.cargo/env\"" >&2; exit 1
fi
if ! command -v unzip >/dev/null; then
    echo "ERRO: instale unzip (sudo apt install unzip)" >&2; exit 1
fi
if ! rustup run nightly rustc --version >/dev/null 2>&1; then
    echo "ERRO: toolchain nightly ausente — rode: rustup toolchain install nightly --profile minimal" >&2; exit 1
fi
if ! cargo fuzz --version >/dev/null 2>&1; then
    echo "ERRO: cargo-fuzz ausente — rode: cargo install cargo-fuzz@0.13.2 --locked" >&2; exit 1
fi
# cargo-fuzz exige nightly; força em TODOS os cargos filhos, mesmo que o
# default do rustup seja stable (não depende de `rustup default nightly`).
export RUSTUP_TOOLCHAIN=nightly

# ---------------------------------------------------------------- seeds ----
# Corpus real > corpus vazio: o APK golden alimenta TODOS os targets.
echo "==> preparando seeds"
if [ ! -s "${APK}" ]; then
    echo "    APK golden ausente; baixando via golden-apks/fetch.sh"
    bash "${REPO}/golden-apks/fetch.sh"
fi
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
# -j = caminhos achatados; classes*.dex cobre classes.dex/classes2/3.dex
unzip -o -j -q "${APK}" 'classes*.dex' 'resources.arsc' 'AndroidManifest.xml' -d "${TMP}"
# res/*.xml: o '*' do unzip cruza diretórios (pega res/layout/*.xml etc.)
( cd "${TMP}" && unzip -o -q "${APK}" 'res/*.xml' 2>/dev/null ) || true

APK_CORPUS="${REPO}/crates/rd-apk/fuzz/corpus"
DEX_CORPUS="${REPO}/crates/rd-dex/fuzz/corpus"
mkdir -p "${APK_CORPUS}/apk_full" "${APK_CORPUS}/zip" "${APK_CORPUS}/axml" \
         "${APK_CORPUS}/arsc" "${DEX_CORPUS}/dex"
cp "${APK}" "${APK_CORPUS}/apk_full/"                 # APK inteiro (ZIP+AXML+ARSC+DEX)
cp "${APK}" "${APK_CORPUS}/zip/"                      # container ZIP real
cp "${TMP}/AndroidManifest.xml" "${APK_CORPUS}/axml/manifest.xml"
cp "${TMP}/resources.arsc" "${APK_CORPUS}/arsc/"
cp "${TMP}"/classes*.dex "${DEX_CORPUS}/dex/"

# NÃO usar `find | head | while` direto: com pipefail, o SIGPIPE do find
# (exit 141) mataria o script no APK real (703 res). Escrever em arquivo e
# consumir depois — `|| true` cobre res/ inexistente e o truncamento do head.
# ATENÇÃO: -size com unidade M/K arredonda PRA CIMA — `-size -1M` só casa
# arquivo VAZIO. Para "< 1 MiB" usar bytes exatos: -size -1048576c.
if [ -d "${TMP}/res" ]; then
    find "${TMP}/res" -name '*.xml' -size -1048576c 2>/dev/null \
        | head -50 > "${TMP}/axml-seeds.txt" || true
    while IFS= read -r f; do
        [ -n "${f}" ] && cp "${f}" "${APK_CORPUS}/axml/" || true
    done < "${TMP}/axml-seeds.txt"
fi

echo "    seeds ok: apk_full=$(ls "${APK_CORPUS}/apk_full" | wc -l) zip=$(ls "${APK_CORPUS}/zip" | wc -l) axml=$(ls "${APK_CORPUS}/axml" | wc -l) arsc=$(ls "${APK_CORPUS}/arsc" | wc -l) dex=$(ls "${DEX_CORPUS}/dex" | wc -l)"

# --------------------------------------------------------------- workers ---
if [ "${WORKERS}" = "0" ]; then
    CORES="$(nproc)"
    WORKERS=$(( CORES / 5 ))
    if [ "${WORKERS}" -lt 1 ]; then WORKERS=1; fi
fi
mkdir -p "${LOGDIR}"
# Proveniência da campanha (vai junto no relatório do DoD)
{
    date -u +"%Y-%m-%dT%H:%M:%SZ"
    echo "repo: ${REPO}"
    echo "duracao_por_target_s: ${SECS}  workers_por_target: ${WORKERS}  cores: $(nproc)"
    rustc --version
    cargo fuzz --version
} > "${LOGDIR}/env.txt" 2>/dev/null || true
echo "==> campanha: 5 targets × ${SECS}s × ${WORKERS} workers/target — logs em ${LOGDIR}"
echo "    (corpus cresce em crates/*/fuzz/corpus; artifacts em crates/*/fuzz/artifacts)"

# Guarda de disco: campanha longa + corpus crescendo exige folga. Aviso (não
# aborta) abaixo de 5 GB livres no filesystem do repo.
FREE_KB="$(df -Pk "${REPO}" | awk 'NR==2 {print $4}' || echo 0)"
if [ "${FREE_KB}" -lt 5242880 ]; then
    echo "AVISO: só $(( FREE_KB / 1048576 )) GB livres em ${REPO} — o corpus pode encher o disco em 24h."
    echo "       Considere liberar espaço; resgate com 'cargo fuzz merge' (veja cabeçalho deste script)."
fi

# ------------------------------------------------------------ execução ----
# max_len ≥ maior seed (senão o libFuzzer TRUNCA a seed — mata o EOCD do ZIP):
#   apk_full/zip: 16 MB · dex: 10 MB · arsc: 4 MB · axml: 2 MB
pids=()
run_target() { # crate target max_len
    local crate="$1" target="$2" max_len="$3"
    mkdir -p "${REPO}/crates/${crate}/fuzz/artifacts"
    (
        cd "${REPO}/crates/${crate}" && cargo fuzz run --release "${target}" -- \
            -max_total_time="${SECS}" \
            -fork="${WORKERS}" \
            -ignore_crashes=1 -ignore_timeouts=1 -ignore_ooms=1 \
            -rss_limit_mb=2048 -malloc_limit_mb=2048 \
            -timeout=25 \
            -max_len="${max_len}" \
            -print_final_stats=1
    ) > "${LOGDIR}/${crate}-${target}.log" 2>&1
    echo "[$(date +%H:%M:%S)] concluído: ${crate}/${target}" >> "${LOGDIR}/done.txt"
}
for spec in "rd-apk apk_full 16777216" "rd-apk zip 16777216" "rd-apk axml 2097152" \
            "rd-apk arsc 4194304" "rd-dex dex 10485760"; do
    # shellcheck disable=SC2086
    set -- ${spec}
    run_target "$1" "$2" "$3" &
    pids+=("$!")
    sleep 2   # espaçar builds iniciais (o cargo serializa o lock, sem erro)
done
for p in "${pids[@]}"; do
    wait "${p}" || fail=$(( fail + 1 ))
done

# ------------------------------------------------------------- relatório ---
echo; echo "================ RELATÓRIO DA CAMPANHA ================"
echo "Logs:            ${LOGDIR}"
echo "Duração/target:  ${SECS}s × ${WORKERS} workers"
for f in "${LOGDIR}"/*.log; do
    [ -e "${f}" ] || continue
    # fork mode não imprime o bloco stat::; o total executado está no último
    # pulso "#N:"; fallback: exec/s do pulso; fallback: n/a
    execs="$(grep -oE '^#[0-9]+:' "${f}" | tail -1 | grep -oE '[0-9]+' || true)"
    if [ -z "${execs}" ]; then
        execs="$(grep -oE 'stat::number_of_executed_units: [0-9]+' "${f}" | tail -1 | grep -oE '[0-9]+$' || true)"
    fi
    if [ -z "${execs}" ]; then
        execs="$(grep -oE 'exec/s: [0-9]+' "${f}" | tail -1 | grep -oE '[0-9]+$' || true)"
    fi
    [ -n "${execs}" ] || execs='n/a (ver log)'
    printf '%-28s %s unidades\n' "$(basename "${f}" .log):" "${execs}"
done
echo
# cargo-fuzz grava artifacts em fuzz/artifacts/<target>/crash-<hash> (com
# SUBDIRETÓRIO por target) — casar por NOME dentro de artifacts/, não por path
# exato, senão o relatório daria "SEM CRASH" falso.
CRASH="$(find "${REPO}/crates" -type f -name 'crash-*' -path '*/fuzz/artifacts/*' 2>/dev/null | wc -l || true)"
OOM="$(find   "${REPO}/crates" -type f -name 'oom-*'    -path '*/fuzz/artifacts/*' 2>/dev/null | wc -l || true)"
TMO="$(find   "${REPO}/crates" -type f -name 'timeout-*' -path '*/fuzz/artifacts/*' 2>/dev/null | wc -l || true)"
echo "crashes: ${CRASH} · oom: ${OOM} · timeouts: ${TMO} · targets com falha: ${fail}"
echo
echo "Uso de disco do corpus (cresceu durante a campanha):"
du -sh "${REPO}/crates/rd-apk/fuzz/corpus"/* "${REPO}/crates/rd-dex/fuzz/corpus"/* 2>/dev/null | sed 's/^/  /' || true
if [ "${fail}" -gt 0 ]; then
    echo; echo "Saída final de cada log com falha (build/erro):"
    for f in "${LOGDIR}"/*.log; do
        [ -e "${f}" ] || continue
        echo "--- ${f} (últimas 5 linhas)"
        tail -5 "${f}" || true
    done
fi
if [ "${CRASH}" -eq 0 ]; then
    echo "RESULTADO: ✅ SEM CRASH — DoD do M1 ('fuzz 24h sem crash') satisfeito"
    if [ "${OOM}" -gt 0 ] || [ "${TMO}" -gt 0 ]; then
        echo "           (⚠ ${OOM} oom + ${TMO} timeout arquivados — investigar depois; não são crash)"
    fi
    exit 0
fi
echo "RESULTADO: ❌ ${CRASH} crash-* — bug P0 (Lei nº 1):"
find "${REPO}/crates" -type f -name 'crash-*' -path '*/fuzz/artifacts/*' 2>/dev/null | sed 's/^/  /'
echo "Reproduzir:   (cd crates/<crate> && cargo fuzz run <target> -- <arquivo-do-artifact>)"
echo "Minimizar:    (cd crates/<crate> && cargo fuzz run <target> -- -minimize_crash=1 -runs=10000 <arquivo>)"
exit 1
