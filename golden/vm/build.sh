#!/usr/bin/env bash
# golden/vm/build.sh — compila os casos e dexa para o harness golden do M2.
#
# Fluxo: fontes Java (cases/*.java) → bytecode 17 (javac se houver JDK; senão
# ecj em JRE) → classes.dex (D8 do r8.jar, --min-api 26 = piso da spec).
# Saída: build/classes/*.class (rodapé do GoldenRunner na JVM) e
#        build/classes.dex (consumido pelo `rd vm exec`).
set -euo pipefail

dir="$(cd "$(dirname "$0")" && pwd)"
cd "$dir"
cache="$dir/cache"
build="$dir/build"

# Ferramentas em dia? (cache vazio → fetch primeiro)
if [ ! -e "$cache/r8.jar" ] || { ! command -v javac >/dev/null 2>&1 && [ ! -e "$cache/ecj.jar" ]; }; then
    ./fetch-tools.sh
fi

# ── 1. javac (JDK) ou ecj (JRE) → bytecode 17 ──────────────────────────────
rm -rf "$build/classes"
mkdir -p "$build/classes"
if command -v javac >/dev/null 2>&1; then
    echo "[build] compilando com javac (--release 17)"
    javac --release 17 -d "$build/classes" cases/*.java
else
    echo "[build] compilando com ecj (fonte/target 17, em JRE)"
    java -jar "$cache/ecj.jar" -17 -nowarn -proc:none -d "$build/classes" cases/*.java
fi

# ── 2. D8 → classes.dex (--min-api 26 = piso API do projeto) ───────────────
rm -f "$build/classes.dex"
echo "[build] dexando com D8 (r8.jar) — --release --min-api 26"
java -cp "$cache/r8.jar" com.android.tools.r8.D8 \
    --release \
    --min-api 26 \
    --output "$build" \
    "$build"/classes/br/rustydroid/vm/*.class

# ── 3. validação ────────────────────────────────────────────────────────────
if [ ! -s "$build/classes.dex" ]; then
    echo "ERRO: build/classes.dex não foi gerado" >&2
    exit 1
fi
echo "[build] ok: build/classes.dex ($(stat -c%s "$build/classes.dex") bytes)"
