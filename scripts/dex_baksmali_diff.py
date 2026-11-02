#!/usr/bin/env python3
"""Differential check: `rd dex disasm` vs baksmali 2.5.2 (M1 DoD).

Closes the "comparador não versionado" gap of M1 (docs/ROADMAP.md). For one
APK it:

1. extracts every `classes*.dex` from the APK;
2. disassembles each dex with baksmali 2.5.2 (pinned by SHA-256);
3. disassembles the same APK with `rd dex disasm <apk> --out DIR`;
4. compares, class by class and method by method:
   - the set of classes (descriptor -> file) on both sides;
   - the method count and method signature per class;
   - the instruction sequence of every method.

The GATE is the opcode sequence: every method must have the same opcodes in
the same order (the M1 criterion "idêntico ao baksmali"). Operand-level
differences are counted and reported separately so a regression in operand
rendering is visible without being hidden behind the opcode gate.

Labels (`:cond_0`, `:goto_1`, ...) and branch targets are normalised before
the operand comparison, because the numbering is a presentation detail.

Usage:
    scripts/dex_baksmali_diff.py --rd target/release/rd \
        --baksmali /path/baksmali-2.5.2.jar --apk golden-apks/app.apk \
        [--report reports/dex-diff/app.json] [--max-report 20]

Exit status: 0 = opcode sequence identical for every method; 1 = mismatch;
2 = usage or tooling error.

Standard library only (runs on the ubuntu-latest runner without pip).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from dataclasses import dataclass, field
from pathlib import Path

BAKSMALI_SHA256 = "d3116248cce4f82ec5a31eb7f95ee75daff42ddf6eed0ab573973dc53fbad2e5"

# Directive-bodied blocks that appear inside a method body. Their content is
# data, not instructions, and must not be read as opcodes.
BLOCK_OPEN = (".packed-switch", ".sparse-switch", ".array-data", ".annotation", ".subannotation")
BLOCK_CLOSE = (".end packed-switch", ".end sparse-switch", ".end array-data", ".end annotation", ".end subannotation")

LABEL_RE = re.compile(r"^:[A-Za-z_][\w]*$")
TARGET_RE = re.compile(r"(:[A-Za-z_][\w]*)")
NUM_RE = re.compile(r"(?<![\w.])-?0x[0-9a-fA-F]+|(?<![\w.])-?\d+(?![\w.])")


@dataclass
class Method:
    header: str
    opcodes: list[str] = field(default_factory=list)
    operands: list[str] = field(default_factory=list)


@dataclass
class ClassInfo:
    path: Path
    methods: list[Method] = field(default_factory=list)


def strip_comment(line: str) -> str:
    # Comments start at '#' outside a string literal. Smali strings are the
    # only place a '#' can legitimately appear, so track quotes.
    out, in_str, esc = [], False, False
    for ch in line:
        if in_str:
            out.append(ch)
            if esc:
                esc = False
            elif ch == "\\":
                esc = True
            elif ch == '"':
                in_str = False
            continue
        if ch == '"':
            in_str = True
            out.append(ch)
        elif ch == "#":
            break
        else:
            out.append(ch)
    return "".join(out).strip()


def normalise_operands(text: str) -> str:
    # baksmali escapes a single quote inside a string literal (\') while rd
    # writes it bare; both are the same smali string. Escape pairs are consumed
    # left to right so `\\'` (backslash + quote) is left alone.
    text = re.sub(r"\\(.)", lambda m: "'" if m.group(1) == "'" else m.group(0), text)
    text = TARGET_RE.sub(":L", text)
    text = NUM_RE.sub("N", text)
    return re.sub(r"\s+", " ", text).strip()


def parse_smali(path: Path) -> ClassInfo:
    info = ClassInfo(path=path)
    current: Method | None = None
    in_block = False
    with path.open("r", encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            line = strip_comment(raw)
            if not line:
                continue
            if current is None:
                if line.startswith(".method "):
                    current = Method(header=line)
                continue
            if in_block:
                if line.startswith(BLOCK_CLOSE):
                    in_block = False
                continue
            if line.startswith(".end method"):
                info.methods.append(current)
                current = None
                continue
            if line.startswith(BLOCK_OPEN):
                in_block = True
                continue
            if line.startswith(".") or LABEL_RE.match(line):
                # .line, .registers, .param, .catch, .locals, .prologue, labels…
                continue
            opcode, _, rest = line.partition(" ")
            current.opcodes.append(opcode)
            current.operands.append(normalise_operands(rest))
    return info


def index_tree(root: Path, prefix: str = "") -> dict[str, Path]:
    """Map class descriptor (`Lpkg/Cls;`) -> smali file, relative to root.

    baksmali lays out `pkg/Cls.smali` (prefix "L" is added to rebuild the
    descriptor); `rd dex disasm` already writes `Lpkg/Cls.smali`. Both end up
    as the same descriptor string.
    """
    out: dict[str, Path] = {}
    for p in root.rglob("*.smali"):
        rel = p.relative_to(root).as_posix()
        out[prefix + rel[:-len(".smali")] + ";"] = p
    return out


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def run(cmd: list[str], cwd: Path | None = None) -> None:
    proc = subprocess.run(cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"{' '.join(cmd)} failed ({proc.returncode}):\n{proc.stderr[-2000:]}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--rd", required=True, type=Path, help="path to the rd binary")
    ap.add_argument("--baksmali", required=True, type=Path, help="path to baksmali-2.5.2.jar")
    ap.add_argument("--apk", required=True, type=Path)
    ap.add_argument("--java", default="java")
    ap.add_argument("--report", type=Path, help="write a JSON report here")
    ap.add_argument("--max-report", type=int, default=20, help="mismatches listed in the report")
    ap.add_argument("--keep", action="store_true", help="keep the temporary directory")
    args = ap.parse_args()

    if not args.apk.is_file():
        print(f"error: APK not found: {args.apk}", file=sys.stderr)
        return 2
    if not args.rd.is_file():
        print(f"error: rd binary not found: {args.rd}", file=sys.stderr)
        return 2
    if not args.baksmali.is_file():
        print(f"error: baksmali jar not found: {args.baksmali}", file=sys.stderr)
        return 2
    digest = sha256(args.baksmali)
    if digest != BAKSMALI_SHA256:
        print(f"error: baksmali sha256 mismatch: {digest} (expected {BAKSMALI_SHA256})", file=sys.stderr)
        return 2

    tmp = Path(tempfile.mkdtemp(prefix="rd-dexdiff-"))
    try:
        dex_dir = tmp / "dex"
        bs_dir = tmp / "baksmali"
        rd_dir = tmp / "rd"
        dex_dir.mkdir()
        bs_dir.mkdir()

        with zipfile.ZipFile(args.apk) as zf:
            dex_names = sorted(n for n in zf.namelist() if re.fullmatch(r"classes\d*\.dex", n))
            if not dex_names:
                print("error: APK has no classes*.dex", file=sys.stderr)
                return 2
            for name in dex_names:
                zf.extract(name, dex_dir)

        for name in dex_names:
            out = bs_dir / Path(name).stem
            run([args.java, "-jar", str(args.baksmali), "disassemble", str(dex_dir / name), "-o", str(out)])

        run([str(args.rd), "dex", "disasm", str(args.apk), "--out", str(rd_dir)])

        # baksmali writes one sub-directory per dex; descriptors are relative
        # to that sub-directory. A class defined twice is a tooling error.
        bs_index: dict[str, Path] = {}
        for sub in sorted(p for p in bs_dir.iterdir() if p.is_dir()):
            for desc, path in index_tree(sub, prefix="L").items():
                if desc in bs_index:
                    print(f"error: class {desc} emitted by two dex files", file=sys.stderr)
                    return 2
                bs_index[desc] = path
        rd_index = index_tree(rd_dir)
        only_bs = sorted(set(bs_index) - set(rd_index))
        only_rd = sorted(set(rd_index) - set(bs_index))

        classes_compared = 0
        methods_compared = 0
        methods_opcode_equal = 0
        methods_operand_equal = 0
        method_count_mismatch: list[str] = []
        opcode_mismatch: list[dict] = []
        operand_mismatch = 0
        operand_samples: list[dict] = []

        for desc in sorted(set(bs_index) & set(rd_index)):
            bs_cls = parse_smali(bs_index[desc])
            rd_cls = parse_smali(rd_index[desc])
            classes_compared += 1
            if len(bs_cls.methods) != len(rd_cls.methods):
                method_count_mismatch.append(desc)
                continue
            for bm, rm in zip(bs_cls.methods, rd_cls.methods):
                methods_compared += 1
                if bm.opcodes == rm.opcodes:
                    methods_opcode_equal += 1
                    if bm.operands == rm.operands:
                        methods_operand_equal += 1
                    else:
                        operand_mismatch += 1
                        if len(operand_samples) < args.max_report:
                            diff = next(
                                (i for i, (a, b) in enumerate(zip(bm.operands, rm.operands)) if a != b),
                                0,
                            )
                            operand_samples.append({
                                "class": desc,
                                "method": bm.header,
                                "index": diff,
                                "baksmali": f"{bm.opcodes[diff]} {bm.operands[diff]}",
                                "rd": f"{rm.opcodes[diff]} {rm.operands[diff]}",
                            })
                else:
                    if len(opcode_mismatch) < args.max_report:
                        first = next(
                            (i for i, (a, b) in enumerate(zip(bm.opcodes, rm.opcodes)) if a != b),
                            min(len(bm.opcodes), len(rm.opcodes)),
                        )
                        opcode_mismatch.append({
                            "class": desc,
                            "method": bm.header,
                            "index": first,
                            "baksmali": bm.opcodes[first:first + 3],
                            "rd": rm.opcodes[first:first + 3],
                        })

        ok = (
            not only_bs and not only_rd and not method_count_mismatch
            and not opcode_mismatch and methods_opcode_equal == methods_compared
        )
        summary = {
            "apk": args.apk.name,
            "apk_sha256": sha256(args.apk),
            "baksmali_sha256": digest,
            "dex_files": dex_names,
            "classes_baksmali": len(bs_index),
            "classes_rd": len(rd_index),
            "classes_only_in_baksmali": len(only_bs),
            "classes_only_in_rd": len(only_rd),
            "classes_compared": classes_compared,
            "methods_compared": methods_compared,
            "methods_opcode_identical": methods_opcode_equal,
            "methods_operand_identical": methods_operand_equal,
            "methods_operand_mismatch": operand_mismatch,
            "classes_method_count_mismatch": len(method_count_mismatch),
            "gate_passed": ok,
            "samples": {
                "only_in_baksmali": only_bs[:args.max_report],
                "only_in_rd": only_rd[:args.max_report],
                "method_count_mismatch": method_count_mismatch[:args.max_report],
                "opcode_mismatch": opcode_mismatch,
                "operand_mismatch": operand_samples,
            },
        }
        print(json.dumps({k: v for k, v in summary.items() if k != "samples"}, indent=2))
        if args.report:
            args.report.parent.mkdir(parents=True, exist_ok=True)
            args.report.write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
        if not ok:
            print("GATE FAILED: opcode sequences differ from baksmali 2.5.2", file=sys.stderr)
            return 1
        print("GATE PASSED: every method's opcode sequence matches baksmali 2.5.2")
        return 0
    finally:
        if args.keep:
            print(f"kept: {tmp}", file=sys.stderr)
        else:
            shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
