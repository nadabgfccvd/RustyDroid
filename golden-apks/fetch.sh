#!/usr/bin/env bash
#
# golden-apks/fetch.sh — download helper for the Golden APK suite (spec GH-05).
#
# GH-05 POLICY: **no binaries are ever committed to this repository.**
# The .gitignore covers golden-apks/*.apk (and cache/); this script is the only
# sanctioned way to materialize the golden APKs locally / on CI runners.
# Only OSS, freely redistributable APKs are fetched here.
#
# Usage:
#   ./fetch.sh                 # downloads the default F-Droid client APK
#   ./fetch.sh <pkg1> [pkg2…]  # additionally fetches extra F-Droid packages
#   ./fetch.sh --list          # lists present APKs + their sha256
#
# Dependencies: curl, sha256sum. python3 is optional (only used to parse the
# F-Droid package API JSON); without it, extra packages are skipped with a
# warning and the default download still works.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
APK_DIR="${REPO_ROOT}/golden-apks"
CACHE_DIR="${APK_DIR}/cache"

USER_AGENT="RustyDroid-GoldenAPK-fetcher/0.1 (+https://github.com/nadabgfccvd/RustyDroid)"
FDROID_BASE="https://f-droid.org"

mkdir -p "${CACHE_DIR}"

fetch() {
    local url="$1"
    local out="$2"
    local min_bytes="$3"

    echo "==> ${url}"
    curl -L --fail --silent --show-error \
        --user-agent "${USER_AGENT}" \
        -o "${out}" \
        "${url}"

    local size
    size="$(wc -c < "${out}")"
    if [ "${size}" -le "${min_bytes}" ]; then
        echo "warning: ${out} is only ${size} bytes (expected > ${min_bytes}); removing" >&2
        rm -f "${out}"
        return 1
    fi
    echo "    saved $(basename -- "${out}") (${size} bytes)"
}

fdroid_version_code() {
    # Prints the suggested versionCode for a package, or nothing on failure.
    local pkg="$1"
    local api_json
    api_json="$(curl -L --fail --silent --show-error \
        --user-agent "${USER_AGENT}" \
        "${FDROID_BASE}/api/v1/packages/${pkg}")" || return 1

    if command -v python3 >/dev/null 2>&1; then
        printf '%s' "${api_json}" | python3 -c '
import json, sys
doc = json.load(sys.stdin)
code = doc.get("suggestedVersionCode")
if not code:
    pkgs = doc.get("packages") or []
    if pkgs:
        code = max(p.get("versionCode", 0) for p in pkgs)
print(int(code) if code else "")
'
    else
        echo "warning: python3 not found; cannot parse F-Droid API JSON for ${pkg}" >&2
        return 1
    fi
}

cmd_list() {
    local found=0
    shopt -s nullglob
    for apk in "${APK_DIR}"/*.apk; do
        found=1
        printf '%-40s %10s  sha256=%s\n' \
            "$(basename -- "${apk}")" \
            "$(wc -c < "${apk}") bytes" \
            "$(sha256sum "${apk}" | cut -d' ' -f1)"
    done
    shopt -u nullglob
    if [ "${found}" -eq 0 ]; then
        echo "no APKs present in ${APK_DIR} — run ./fetch.sh first"
    fi
}

main() {
    if [ "${1:-}" = "--list" ]; then
        cmd_list
        return 0
    fi

    # Default: F-Droid stable client (Apache-2.0) — the canonical golden APK.
    local default_out="${APK_DIR}/org.fdroid.fdroid.apk"
    if [ -s "${default_out}" ]; then
        echo "==> org.fdroid.fdroid.apk already present, skipping (delete to re-fetch)"
    else
        fetch "${FDROID_BASE}/F-Droid.apk" "${default_out}" 1048576 \
            || echo "warning: failed to fetch default F-Droid client APK" >&2
    fi

    # Extra F-Droid packages passed as arguments (fault tolerant per package).
    local pkg code url out
    for pkg in "$@"; do
        case "${pkg}" in
            -*) echo "warning: ignoring unknown option '${pkg}'" >&2; continue ;;
        esac
        if ! code="$(fdroid_version_code "${pkg}")" || [ -z "${code}" ]; then
            echo "warning: could not resolve a version for '${pkg}'; skipping" >&2
            continue
        fi
        url="${FDROID_BASE}/repo/${pkg}_${code}.apk"
        out="${APK_DIR}/${pkg}.apk"
        if [ -s "${out}" ]; then
            echo "==> ${pkg}.apk already present, skipping"
            continue
        fi
        fetch "${url}" "${out}" 1048576 \
            || echo "warning: failed to fetch '${pkg}' (${url}); continuing" >&2
    done

    echo
    echo "Downloaded files (sha256):"
    cmd_list
}

main "$@"
