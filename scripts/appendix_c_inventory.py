#!/usr/bin/env python3
"""Appendix C inventory: count the public Android SDK surface per API level.

Reads the real SDK ``android.jar`` stub for each API level (26-36 by default)
and counts, for every ``[[surface]]`` prefix registered in
``data/api-coverage.toml``, the public and protected classes, constructors,
methods and fields. ``@hide`` / internal framework classes are not present in
the SDK stub jar, so these counts are the public SDK surface.

Only the class-file format is parsed (no javap, no JDK required). The script
also checks which inventoried classes already have a descriptor literal in
``crates/rd-vm/src`` (a class-level, not method-level, implementation signal).

The jars are not committed. Pass the directory that holds one
``android-<level>/android.jar`` per level (see ``--sdk-dir``).

Outputs:
  * ``docs/api-inventory/android-api-surface.csv``  (surface x API level)
  * ``docs/api-inventory/android-api-classes.csv``  (class x API level, flags)
  * ``docs/appendix-c-inventory.md``                (summary tables)

Usage::

    python3 scripts/appendix_c_inventory.py --sdk-dir ~/.cache/android-jars/sdk
"""

from __future__ import annotations

import argparse
import csv
import io
import os
import re
import struct
import sys
import tomllib
import zipfile
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

ACC_PUBLIC = 0x0001
ACC_PRIVATE = 0x0002
ACC_PROTECTED = 0x0004
ACC_BRIDGE = 0x0040
ACC_SYNTHETIC = 0x1000
EXPOSED = ACC_PUBLIC | ACC_PROTECTED

DEFAULT_LEVELS = [26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36]
REPO = Path(__file__).resolve().parent.parent


@dataclass
class ClassInfo:
    name: str  # internal name, e.g. android/util/ArrayMap
    flags: int
    super_name: str | None
    interfaces: list[str]
    methods: list[tuple[int, str, str]] = field(default_factory=list)  # flags, name, desc
    fields: list[tuple[int, str]] = field(default_factory=list)  # flags, name


def _parse_class(data: bytes) -> ClassInfo:
    if data[:4] != b"\xca\xfe\xba\xbe":
        raise ValueError("not a class file")
    pos = 8
    count = struct.unpack_from(">H", data, pos)[0]
    pos += 2
    utf8: dict[int, str] = {}
    class_refs: dict[int, int] = {}
    i = 1
    while i < count:
        tag = data[pos]
        pos += 1
        if tag == 1:  # Utf8
            length = struct.unpack_from(">H", data, pos)[0]
            pos += 2
            utf8[i] = data[pos : pos + length].decode("utf-8", "replace")
            pos += length
        elif tag == 7:  # Class
            class_refs[i] = struct.unpack_from(">H", data, pos)[0]
            pos += 2
        elif tag in (8, 16, 19, 20):  # String, MethodType, Module, Package
            pos += 2
        elif tag in (3, 4, 9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag in (5, 6):  # Long, Double take two slots
            pos += 8
            i += 1
        elif tag == 15:  # MethodHandle
            pos += 3
        else:
            raise ValueError(f"unknown constant tag {tag}")
        i += 1

    access, this_idx, super_idx = struct.unpack_from(">HHH", data, pos)
    pos += 6
    name = utf8[class_refs[this_idx]]
    super_name = utf8[class_refs[super_idx]] if super_idx else None
    n_if = struct.unpack_from(">H", data, pos)[0]
    pos += 2
    interfaces = []
    for _ in range(n_if):
        interfaces.append(utf8[class_refs[struct.unpack_from(">H", data, pos)[0]]])
        pos += 2

    info = ClassInfo(name, access, super_name, interfaces)
    for kind in ("field", "method"):
        n = struct.unpack_from(">H", data, pos)[0]
        pos += 2
        for _ in range(n):
            acc, name_idx, desc_idx, n_attr = struct.unpack_from(">HHHH", data, pos)
            pos += 8
            pos = _skip_attributes(data, pos, n_attr)
            if kind == "field":
                info.fields.append((acc, utf8[name_idx]))
            else:
                info.methods.append((acc, utf8[name_idx], utf8[desc_idx]))
    return info


def _skip_attributes(data: bytes, pos: int, n: int) -> int:
    for _ in range(n):
        length = struct.unpack_from(">I", data, pos + 2)[0]
        pos += 6 + length
    return pos


def load_jar(path: Path) -> dict[str, ClassInfo]:
    classes: dict[str, ClassInfo] = {}
    with zipfile.ZipFile(path) as zf:
        for entry in zf.namelist():
            if not entry.endswith(".class") or entry.startswith("META-INF/"):
                continue
            if entry.endswith("module-info.class") or entry.endswith("package-info.class"):
                continue
            info = _parse_class(zf.read(entry))
            classes[info.name.replace("/", ".")] = info
    return classes


def exposed_members(info: ClassInfo) -> tuple[int, int, int]:
    """Return (constructors, methods, fields) that are public or protected."""
    ctors = methods = fields = 0
    for acc, name, _ in info.methods:
        if acc & ACC_SYNTHETIC or acc & ACC_BRIDGE or not acc & EXPOSED:
            continue
        if name == "<clinit>":
            continue
        if name == "<init>":
            ctors += 1
        else:
            methods += 1
    for acc, _ in info.fields:
        if acc & ACC_SYNTHETIC or not acc & EXPOSED:
            continue
        fields += 1
    return ctors, methods, fields


def package_of(fqcn: str) -> str:
    """Package of a binary name like android.app.Activity$Foo."""
    top = fqcn.split("$", 1)[0]
    return top.rsplit(".", 1)[0] if "." in top else ""


def surface_members(classes: dict[str, ClassInfo], prefix: str) -> list[str]:
    """Classes that belong to a registered prefix.

    A prefix is either a package (``android.util``: exact package match, so
    subpackages are separate registry rows) or a class (``android.os.Bundle``:
    the class and its nested classes).
    """
    out = []
    for fqcn in classes:
        top = fqcn.split("$", 1)[0]
        if top == prefix or fqcn.startswith(prefix + "$"):
            out.append(fqcn)
        elif package_of(fqcn) == prefix and "$" not in fqcn.rsplit(".", 1)[-1]:
            out.append(fqcn)
    return out


def public_top_level(classes: dict[str, ClassInfo]) -> dict[str, ClassInfo]:
    return {
        n: c
        for n, c in classes.items()
        if c.flags & EXPOSED and not c.flags & ACC_SYNTHETIC
    }


def descriptor_literals(src_root: Path) -> set[str]:
    """Descriptors like Landroid/util/ArrayMap; found anywhere in rd-vm sources."""
    found: set[str] = set()
    pattern = re.compile(r"L(android/[A-Za-z0-9_/$]+);")
    for path in (src_root / "crates" / "rd-vm" / "src").rglob("*.rs"):
        for m in pattern.finditer(path.read_text(encoding="utf-8", errors="replace")):
            found.add(m.group(1).replace("/", "."))
    return found


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--sdk-dir", required=True, type=Path,
                    help="directory with android-<level>/android.jar (or *.zip extracts)")
    ap.add_argument("--levels", default=",".join(map(str, DEFAULT_LEVELS)))
    ap.add_argument("--out", type=Path, default=REPO / "docs" / "api-inventory")
    args = ap.parse_args(argv)

    levels = [int(x) for x in args.levels.split(",") if x.strip()]
    registry = tomllib.loads((REPO / "data" / "api-coverage.toml").read_text(encoding="utf-8"))
    surfaces = registry.get("surface", [])
    descs = descriptor_literals(REPO)

    per_level_classes: dict[int, dict[str, ClassInfo]] = {}
    for level in levels:
        jar = args.sdk_dir / f"android-{level}" / "android.jar"
        if not jar.exists():
            print(f"missing {jar}", file=sys.stderr)
            continue
        per_level_classes[level] = load_jar(jar)
        print(f"API {level}: {len(per_level_classes[level])} class files", file=sys.stderr)

    loaded = sorted(per_level_classes)
    if not loaded:
        print("no android.jar found", file=sys.stderr)
        return 2

    args.out.mkdir(parents=True, exist_ok=True)

    # Surface rows: one per registered prefix per level.
    surface_rows = []
    surface_summary: dict[str, dict[int, tuple[int, int, int]]] = {}
    for s in surfaces:
        prefix = s["prefix"]
        sid = s["id"]
        status = s.get("status", "")
        surface_summary[sid] = {}
        for level in loaded:
            classes = per_level_classes[level]
            names = surface_members(classes, prefix)
            cls_count = len([n for n in names if classes[n].flags & EXPOSED])
            c = m = f = 0
            for n in names:
                if not classes[n].flags & EXPOSED:
                    continue
                cc, mm, ff = exposed_members(classes[n])
                c += cc
                m += mm
                f += ff
            implemented = len([n for n in names if classes[n].flags & EXPOSED and n in descs])
            surface_summary[sid][level] = (cls_count, c + m + f, implemented)
            surface_rows.append({
                "surface_id": sid,
                "prefix": prefix,
                "registry_status": status,
                "api_level": level,
                "public_classes": cls_count,
                "constructors": c,
                "methods": m,
                "fields": f,
                "members_total": c + m + f,
                "classes_with_rd_vm_descriptor": implemented,
            })

    with (args.out / "android-api-surface.csv").open("w", newline="", encoding="utf-8") as fh:
        writer = csv.DictWriter(fh, fieldnames=list(surface_rows[0].keys()))
        writer.writeheader()
        writer.writerows(surface_rows)

    # Full class inventory (flat), for diffing between API levels.
    class_rows = []
    all_names: set[str] = set()
    for level in loaded:
        all_names.update(public_top_level(per_level_classes[level]).keys())
    for name in sorted(all_names):
        row = {"class": name, "package": package_of(name)}
        for level in loaded:
            info = per_level_classes[level].get(name)
            if info is None or not info.flags & EXPOSED:
                row[f"api{level}"] = ""
            else:
                cc, mm, ff = exposed_members(info)
                row[f"api{level}"] = f"{cc}/{mm}/{ff}"
        row["in_rd_vm"] = "yes" if name in descs else ""
        class_rows.append(row)
    with (args.out / "android-api-classes.csv").open("w", newline="", encoding="utf-8") as fh:
        fieldnames = ["class", "package"] + [f"api{l}" for l in loaded] + ["in_rd_vm"]
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(class_rows)

    # Unregistered packages: public surface that no registry prefix covers.
    registered = [s["prefix"] for s in surfaces]
    pkg_count: dict[str, int] = defaultdict(int)
    for name in public_top_level(per_level_classes[loaded[-1]]):
        if not name.startswith("android."):
            continue  # java.*/javax.* are outside the android.* registry scope
        if any(name == r or name.startswith(r + "$") or package_of(name) == r for r in registered):
            continue
        pkg_count[package_of(name)] += 1

    write_readme(REPO / "docs" / "appendix-c-inventory.md", loaded, surfaces, surface_summary, per_level_classes,
                 pkg_count, descs)
    print(f"wrote {args.out}", file=sys.stderr)
    return 0


def write_readme(summary_path: Path, loaded: list[int], surfaces: list[dict],
                 summary: dict[str, dict[int, tuple[int, int, int]]],
                 per_level: dict[int, dict[str, ClassInfo]], uncovered: dict[str, int],
                 descs: set[str]) -> None:
    lines: list[str] = []
    w = lines.append
    w("# Appendix C — public Android API inventory (API 26–36)")
    w("")
    w("Generated by `scripts/appendix_c_inventory.py` from the official SDK")
    w("`android.jar` of each platform level (`platform-NN` packages from")
    w("`dl.google.com/android/repository`). Counts are the **public + protected**")
    w("SDK surface only; synthetic and bridge methods are excluded.")
    w("")
    w("This is an **inventory**, not a completion claim. Under the rule in")
    w("[`appendix-c-literal-completion.md`](appendix-c-literal-completion.md), every")
    w("counted item not implemented and tested is an open blocker.")
    w("")
    w("## Totals per API level")
    w("")
    w("| API | public classes (incl. nested) | public members (ctor+method+field) | `android.*` classes with an `rd-vm` descriptor literal |")
    w("|---:|---:|---:|---:|")
    for level in loaded:
        classes = public_top_level(per_level[level])
        members = 0
        for name, info in classes.items():
            cc, mm, ff = exposed_members(info)
            members += cc + mm + ff
        covered = len([n for n in classes if n in descs])
        w(f"| {level} | {len(classes)} | {members} | {covered} |")
    w("")
    w("## Registered surfaces (`data/api-coverage.toml`)")
    w("")
    head = "| surface | prefix | status | " + " | ".join(f"API {l}" for l in loaded) + " |"
    w(head)
    w("|---|---|---|" + "---:|" * len(loaded))
    for s in surfaces:
        sid = s["id"]
        cells = []
        for level in loaded:
            cls_count, members, _ = summary[sid][level]
            cells.append(f"{cls_count} cls / {members} mem")
        w(f"| `{sid}` | `{s['prefix']}` | {s.get('status', '')} | " + " | ".join(cells) + " |")
    w("")
    w("Counts are `public classes / public members` at each API level. The per-surface")
    w("implementation signal (`classes_with_rd_vm_descriptor`) is in")
    w("[`api-inventory/android-api-surface.csv`](api-inventory/android-api-surface.csv).")
    w("")
    w("## Public packages not covered by any registered surface")
    w("")
    w(f"Public `android.*` classes in API {loaded[-1]} per package, for packages that no")
    w("`[[surface]]` prefix names (registry gaps). `java.*`/`javax.*` are not in this")
    w("registry's scope.")
    w("")
    w("| package | public classes |")
    w("|---|---:|")
    for pkg, n in sorted(uncovered.items(), key=lambda kv: (-kv[1], kv[0])):
        w(f"| `{pkg}` | {n} |")
    w("")
    w("## Method")
    w("")
    w("- Source: SDK `android.jar` per level, class files parsed directly.")
    w("- Surface membership: a prefix that is a package matches exact package only;")
    w("  a prefix that is a class matches the class and its nested classes.")
    w("- `rd-vm descriptor literals` counts classes whose `Landroid/...;` descriptor")
    w("  appears in `crates/rd-vm/src`. It is a class-level signal only and does **not**")
    w("  prove that methods or behaviour are implemented.")
    w("- API 34 uses `platform-34-ext7` (`ApiLevel=34`, `ExtensionLevel=7`, `IsBaseSdk=true`)")
    w("  because no plain `platform-34` package is published on `dl.google.com`.")
    w("")
    summary_path.write_text("\n".join(lines), encoding="utf-8")


if __name__ == "__main__":
    raise SystemExit(main())
