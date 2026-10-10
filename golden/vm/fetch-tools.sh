#!/usr/bin/env bash
# golden/vm/fetch-tools.sh — baixa as ferramentas do pipeline golden do M2.
#
# Papel: providenciar compilador Java → bytecode e bytecode → DEX sem exigir
# JDK instalado (o ambiente local tem só JRE). Tudo cai em cache/, que é
# gitignored (política GH-05: NENHUM binário/jar é commitado).
#
# Ferramentas:
# - ecj (Eclipse Compiler for Java, roda em JRE puro) — só se `javac` não
#   existir no PATH (CI do GitHub tem JDK; local usa ecj).
#   Fonte: Maven Central  org/eclipse/jdt/ecj/<VER>/ecj-<VER>.jar
# - r8 (contém o D8, dexer) — sempre necessário.
#   Fonte: Google Maven     com/android/tools/r8/<VER>/r8-<VER>.jar
#   (o r8 NÃO é publicado no repo1.maven.org — o home oficial é o
#   maven.google.com/dl.google.com; validado com curl -sI em 2026-02.)
#
# Versões pinnadas (validadas com curl -sI → 200; se 404, tentamos vizinhas):
#   ecj 3.42.0 (fallback 3.41.0, 3.40.0)
#   r8  8.5.35 (fallback 8.3.37)

set -euo pipefail

dir="$(cd "$(dirname "$0")" && pwd)"
cache="$dir/cache"
mkdir -p "$cache"
UA="RustyDroid-golden/1.0 (+https://github.com/nadabgfccvd/RustyDroid)"

# fetch <url> <arquivo-destino> <tamanho-mínimo>
# Baixa com curl (falha em 404/erro HTTP), valida tamanho e promove o arquivo.
fetch() {
    local url="$1" out="$2" min_size="$3"
    if [ -s "$out" ] && [ "$(stat -c%s "$out")" -ge "$min_size" ]; then
        echo "[fetch-tools] ok (cache): $out"
        return 0
    fi
    echo "[fetch-tools] baixando: $url"
    local tmp
    tmp="$(mktemp "${out}.part.XXXXXX")"
    if ! curl -fsSL --retry 3 --retry-delay 2 -A "$UA" -o "$tmp" "$url"; then
        echo "[fetch-tools] falha no download: $url" >&2
        rm -f "$tmp"
        return 1
    fi
    local size
    size="$(stat -c%s "$tmp")"
    if [ "$size" -lt "$min_size" ]; then
        echo "[fetch-tools] arquivo pequeno demais (${size}B): $url" >&2
        rm -f "$tmp"
        return 1
    fi
    mv "$tmp" "$out"
    echo "[fetch-tools] salvo (${size} bytes): $out"
}

# ── ecj (compilador Java em JRE) ────────────────────────────────────────────
if command -v javac >/dev/null 2>&1; then
    echo "[fetch-tools] javac disponível no PATH — download do ecj pulado"
else
    ecj_ok=0
    for v in 3.42.0 3.41.0 3.40.0; do
        if fetch "https://repo1.maven.org/maven2/org/eclipse/jdt/ecj/${v}/ecj-${v}.jar" \
                 "$cache/ecj-${v}.jar" 2000000; then
            ln -sfn "ecj-${v}.jar" "$cache/ecj.jar"
            ecj_ok=1
            break
        fi
        echo "[fetch-tools] aviso: ecj ${v} indisponível — tentando versão vizinha"
    done
    if [ "$ecj_ok" -ne 1 ] || [ ! -e "$cache/ecj.jar" ]; then
        echo "ERRO: nenhum ecj disponível (tentadas: 3.42.0/3.41.0/3.40.0)" >&2
        exit 1
    fi
fi

# ── r8 (dexer D8) ───────────────────────────────────────────────────────────
r8_ok=0
for v in 8.5.35 8.3.37; do
    if fetch "https://dl.google.com/android/maven2/com/android/tools/r8/${v}/r8-${v}.jar" \
             "$cache/r8-${v}.jar" 1000000; then
        ln -sfn "r8-${v}.jar" "$cache/r8.jar"
        r8_ok=1
        break
    fi
    echo "[fetch-tools] aviso: r8 ${v} indisponível — tentando versão vizinha"
done
if [ "$r8_ok" -ne 1 ] || [ ! -e "$cache/r8.jar" ]; then
    echo "ERRO: nenhum r8 disponível (tentadas: 8.5.35/8.3.37)" >&2
    exit 1
fi

echo "[fetch-tools] ferramentas prontas em $cache"
