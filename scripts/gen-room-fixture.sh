#!/usr/bin/env bash
# Regenera o DEX do fixture de Room (Apêndice D, ST-02): Room 2.6.1 processado
# pelo javac real (annotation processor do room-compiler), depois D8 (R8 8.7.18)
# com as bibliotecas de runtime. Fontes: crates/rd-vm/tests/fixtures/room/src.
#
# Saída: $OUT (padrão: target/room-fixture/classes.dex). O DEX NÃO é versionado:
# A regra vem do RD Sentinel, retirado pela ADR 010 (ver docs/adr/036). O teste
# `crates/rd-vm/tests/jetpack_room.rs` lê o caminho em RD_ROOM_FIXTURE_DEX.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="${WORK:-$ROOT/target/room-fixture}"
OUT="${OUT:-$WORK/classes.dex}"
mkdir -p "$WORK"
cd "$WORK"

# 1. Dependências Maven (Google Maven primeiro; AndroidX não existe no Maven Central).
python3 "$ROOT/scripts/maven_resolve.py" jars \
  androidx.room:room-compiler:2.6.1 androidx.room:room-runtime:2.6.1 \
  androidx.room:room-migration:2.6.1 androidx.sqlite:sqlite-framework:2.4.0
python3 "$ROOT/scripts/maven_resolve.py" jarsrt \
  androidx.room:room-runtime:2.6.1 androidx.room:room-migration:2.6.1 \
  androidx.sqlite:sqlite-framework:2.4.0

# 2. android.jar (platform-35, mesmo pacote do Apêndice C).
if [ ! -f android.jar ]; then
  curl -sSfL -o platform35.zip https://dl.google.com/android/repository/platform-35_r02.zip
  python3 -c "
import zipfile
z = zipfile.ZipFile('platform35.zip')
open('android.jar', 'wb').write(z.read('android-35/android.jar'))"
  rm -f platform35.zip
fi
[ -f r8.jar ] || curl -sSfL -o r8.jar https://dl.google.com/dl/android/maven2/com/android/tools/r8/8.7.18/r8-8.7.18.jar

# 3. javac com o processador do Room; sem dependências duplicadas por artefato.
rm -rf out && mkdir -p out/classes out/gen
PROC=$(ls jars/*.jar | tr '\n' ':')
RT=$(ls jarsrt/*.jar | tr '\n' ':')
DEDUP=$(python3 - jarsrt <<'PY'
import glob, os, re, sys
best = {}
for j in glob.glob(os.path.join(sys.argv[1], '*.jar')):
    name = os.path.basename(j)
    m = re.match(r'(.+?)-(\d[\d.]*(?:-[\w.]+)?)\.jar$', name)
    base = m.group(1) if m else name
    if base not in best or name > os.path.basename(best[base]):
        best[base] = j
print(' '.join(sorted(best.values())))
PY
)
javac -encoding UTF-8 --release 8 -Xlint:-options -d out/classes -s out/gen \
  -processorpath "$PROC" -cp "android.jar:$RT" \
  $(find "$ROOT/crates/rd-vm/tests/fixtures/room/src" -name '*.java')

# 4. D8 (R8 8.7.18), minSdk 26: app + runtime do Room.
rm -rf out/dex && mkdir -p out/dex
java -cp r8.jar com.android.tools.r8.D8 --release --min-api 26 --lib android.jar \
  --output out/dex $(find out/classes -name '*.class') $DEDUP
mkdir -p "$(dirname "$OUT")"
cp out/dex/classes.dex "$OUT"
echo "ok: $OUT ($(sha256sum "$OUT" | cut -d' ' -f1))"
