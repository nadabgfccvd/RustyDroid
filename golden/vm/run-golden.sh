#!/usr/bin/env bash
# golden/vm/run-golden.sh — executa os vetores golden do M2: JVM real vs rd VM.
#
# Passo 1: GoldenRunner na JVM real (java -cp build/classes) → build/expected.tsv
# Passo 2: para cada linha do TSV, `rd vm exec build/classes.dex --class
#          Lbr/rustydroid/vm/<Classe>; --method <m> --sig <s> --args <a> --json`
#          (contrato 5-b: {"status":"ok","result":{"type":..,"value":..}} com
#          value STRING; exceção → {"status":"exception","class","message"}).
# Passo 3: comparação campo a campo (python3 embutido):
#          - int/long/float/double/byte/short: string igual (float/double no
#            formato Java dos DOIS lados);
#          - boolean: VM devolve "1"/"0" (int cru no JSON) → normalizado p/
#            true/false; char: VM devolve o código → normalizado p/ o caractere;
#          - void: VM result null ↔ runner <void>;
#          - exceção: compara APENAS a classe (a mensagem é texto livre e
#            difere entre ART/JVM e a VM mínima — decisão documentada no
#            README; o conteúdo de mensagens é coberto pelos casos
#            customMsgLen/customHash, que passam por length/hashCode).
#
# Uso: ./run-golden.sh [caminho-do-rd]
#      (default: ../../target/release/rd relativo a este script)
# Saída: build/actual.jsonl (respostas brutas) + relatório no stdout.
# Exit 0 SOMENTE se 100% dos casos derem PASS.
set -euo pipefail

dir="$(cd "$(dirname "$0")" && pwd)"
build="$dir/build"
RD_BIN="${1:-$dir/../../target/release/rd}"
DEX="$build/classes.dex"
CLASSES="$build/classes"

if [ ! -x "$RD_BIN" ]; then
    echo "ERRO: binário rd não encontrado/executável: $RD_BIN" >&2
    echo "      passe o caminho como \$1 (ex.: ./run-golden.sh ../../target/release/rd)" >&2
    exit 2
fi
if [ ! -e "$DEX" ] || [ ! -d "$CLASSES" ]; then
    echo "[run-golden] build ausente — rodando ./build.sh primeiro"
    "$dir/build.sh"
fi

echo "[run-golden] binário: $RD_BIN"
echo "[run-golden] passo 1: GoldenRunner na JVM real → build/expected.tsv"
java -cp "$CLASSES" br.rustydroid.vm.GoldenRunner > "$build/expected.tsv"
n_cases=$(wc -l < "$build/expected.tsv")
echo "[run-golden] ${n_cases} vetores gerados"

echo "[run-golden] passo 2+3: rd vm exec por caso + comparação"
python3 - "$RD_BIN" "$DEX" "$build" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

rd, dex, build = sys.argv[1], sys.argv[2], Path(sys.argv[3])
tsv = (build / "expected.tsv").read_text(encoding="utf-8").splitlines()
actual_path = build / "actual.jsonl"
report_path = build / "report.txt"

def norm_vm_value(ret, val):
    """Normaliza o value da VM (JSON) para a forma do runner JVM."""
    if ret == "Z":
        return {"1": "true", "0": "false"}.get(val, val)
    if ret == "C":
        try:
            return chr(int(val))
        except (ValueError, OverflowError):
            return "<char?%s>" % val
    return val  # I/J/F/D/B/S: mesma representação textual

def ret_of(sig):
    return sig[sig.rindex(")") + 1:]

passes, fails, lines = 0, 0, []
with actual_path.open("w", encoding="utf-8") as actual:
    for line in tsv:
        parts = line.split("|", 4)
        if len(parts) != 5:
            fails += 1
            lines.append(f"FAIL <tsv malformado>: {line!r}")
            continue
        cls, met, sig, args, expected = parts
        cmd = [rd, "vm", "exec", dex,
               "--class", f"Lbr/rustydroid/vm/{cls};",
               "--method", met,
               "--sig", sig,
               "--json"]
        if args:
            # sintaxe --args=VALOR: args começando com '-' (ex.: '-7') seriam
            # interpretados como flag pelo clap na forma separada '--args -7'
            cmd += ["--args=" + args]
        name = f"{cls}.{met}{sig}"
        try:
            proc = subprocess.run(cmd, capture_output=True, text=True,
                                  timeout=120)
        except subprocess.TimeoutExpired:
            fails += 1
            lines.append(f"FAIL {name}: TIMEOUT (120s)")
            continue
        raw = proc.stdout.strip()
        actual.write(raw + "\n")
        got = None
        if raw:
            try:
                got = json.loads(raw)
            except json.JSONDecodeError:
                pass
        if got is None:
            fails += 1
            lines.append(f"FAIL {name}: saída não-JSON (rc={proc.returncode}) {raw!r} {proc.stderr.strip()!r}")
            continue

        status = got.get("status")
        # ── exceção esperada: compara APENAS a classe (msg é texto livre) ──
        if expected.startswith("EXCEPTION:"):
            want_cls = expected.split(":", 2)[1]
            if status == "exception":
                got_cls = got.get("class", "")
                if got_cls == want_cls:
                    passes += 1
                    lines.append(f"PASS {name} (exceção {want_cls})")
                else:
                    fails += 1
                    lines.append(f"FAIL {name}: classe de exceção esperada {want_cls}, VM devolveu {got_cls} (msg {got.get('message','')!r})")
            else:
                fails += 1
                lines.append(f"FAIL {name}: esperava EXCEPTION {want_cls}, VM devolveu status={status} {got}")
            continue

        # ── resultado normal ────────────────────────────────────────────────
        if status != "ok":
            fails += 1
            if status == "error":
                lines.append(f"FAIL {name}: erro VM {got.get('code')}: {got.get('cause')}")
            else:
                lines.append(f"FAIL {name}: esperava resultado, VM devolveu {got}")
            continue
        result = got.get("result")
        if expected == "<void>":
            # contrato 5-b: void → result nulo; a implementação devolve
            # {"type":"null","value":null} — ambos os formatos aceitos
            if result is None or result.get("type") == "null":
                passes += 1
                lines.append(f"PASS {name} (<void>)")
            else:
                fails += 1
                lines.append(f"FAIL {name}: esperava <void>, VM devolveu {result}")
            continue
        if result is None or not isinstance(result.get("value"), str):
            fails += 1
            lines.append(f"FAIL {name}: esperava {expected!r}, VM devolveu {result}")
            continue
        got_val = norm_vm_value(ret_of(sig), result["value"])
        if got_val == expected:
            passes += 1
            lines.append(f"PASS {name} = {expected}")
        else:
            fails += 1
            lines.append(f"FAIL {name}: esperado {expected!r} (tipo {ret_of(sig)}), VM {got_val!r} (type {result.get('type')})")

total = passes + fails
summary = f"RESUMO GOLDEN M2: {passes}/{total} PASS, {fails} FAIL"
report_path.write_text("\n".join(lines) + "\n" + summary + "\n", encoding="utf-8")
print("\n".join(lines))
print(summary)
sys.exit(0 if fails == 0 else 1)
PY
rc=$?
if [ "$rc" -eq 0 ]; then
    echo "[run-golden] TODOS os vetores PASS — DoD do M2 satisfeito neste pipeline"
else
    echo "[run-golden] FALHAS acima — detalhe completo em build/report.txt" >&2
fi
exit "$rc"
