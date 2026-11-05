#!/usr/bin/env bash
# Regenera crates/rd-vm/tests/fixtures/rxjava3/classes.dex (Apêndice D, S12).
# Entradas e ferramenta com SHA-256 fixos; falha se qualquer hash diferir.
# Uso: scripts/gen-rxjava3-fixture.sh   (precisa de java e de rede para o Maven)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIX="$ROOT/crates/rd-vm/tests/fixtures/rxjava3"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

R8_URL=https://dl.google.com/dl/android/maven2/com/android/tools/r8/8.7.18/r8-8.7.18.jar
RX_URL=https://repo1.maven.org/maven2/io/reactivex/rxjava3/rxjava/3.1.10/rxjava-3.1.10.jar
RS_URL=https://repo1.maven.org/maven2/org/reactivestreams/reactive-streams/1.0.4/reactive-streams-1.0.4.jar
R8_SHA=58366f77067207c39a17d469de7b05701d2877212a9c55201bcb0af43e59e903
RX_SHA=e9f256f9e805572dd5fd45b140db360d7dc444d0c383049b2d3d7ebf05d84aab
RS_SHA=f75ca597789b3dac58f61857b9ac2e1034a68fa672db35055a8fb4509e325f28
OUT_SHA=4023c2291dd57bf6d7718da62f0b837f3fa7b45555c3ee04550c4f9f284c3fac

fetch() { curl -sSfL --max-time 300 -o "$WORK/$2" "$1"; echo "$3  $WORK/$2" | sha256sum -c --quiet; }
fetch "$R8_URL" r8.jar "$R8_SHA"
fetch "$RX_URL" rxjava.jar "$RX_SHA"
fetch "$RS_URL" rs.jar "$RS_SHA"
cp "$FIX/keep.pro" "$WORK/keep.pro"

mkdir -p "$WORK/out"
java -cp "$WORK/r8.jar" com.android.tools.r8.R8 --release --min-api 26 \
  --output "$WORK/out" --pg-conf "$WORK/keep.pro" "$WORK/rxjava.jar" "$WORK/rs.jar" >/dev/null
echo "$OUT_SHA  $WORK/out/classes.dex" | sha256sum -c --quiet
cp "$WORK/out/classes.dex" "$FIX/classes.dex"
echo "ok: $FIX/classes.dex ($OUT_SHA)"
