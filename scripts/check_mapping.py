#!/usr/bin/env python3
"""
Scan `crates/` for `// JAVA: fully.qualified.Class#method` annotations and generate `MAPPING.md`.
Optionally compares against methods discovered in `reference/decompiled/` (or `reference/server_mappings.txt`).
"""

from __future__ import annotations

import argparse
import re
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES_DIR = ROOT / "crates"
MAPPING_MD = ROOT / "MAPPING.md"
JAVA_ANN_RE = re.compile(r"//\s*JAVA:\s*([A-Za-z0-9_.$#<>]+)")


def scan_rust_annotations() -> dict[str, list[tuple[str, int, str]]]:
    mapping: dict[str, list[tuple[str, int, str]]] = defaultdict(list)
    for rs in sorted(CRATES_DIR.rglob("*.rs")):
        rel = rs.relative_to(ROOT).as_posix()
        lines = rs.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(lines, start=1):
            m = JAVA_ANN_RE.search(line)
            if m:
                java_sym = m.group(1)
                fn_name = ""
                for look in lines[idx : min(len(lines), idx + 6)]:
                    fm = re.search(r"\bfn\s+([A-Za-z0-9_]+)", look)
                    if fm:
                        fn_name = fm.group(1)
                        break
                mapping[java_sym].append((rel, idx, fn_name or "<item>"))
    return dict(mapping)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--list-unmapped", action="store_true", help="Print sample unmapped methods from server_mappings.txt")
    args = parser.parse_args()

    mapping = scan_rust_annotations()
    lines = [
        "# Java-to-Rust Symbol Mapping (`MAPPING.md`)",
        "",
        f"Total mapped Java symbols: **{len(mapping)}**",
        "",
        "| Java Symbol (`Class#method`) | Rust File | Line | Rust Symbol |",
        "|---|---|---:|---|",
    ]
    for java_sym in sorted(mapping):
        for rel, line_no, fn_name in mapping[java_sym]:
            lines.append(f"| `{java_sym}` | `{rel}` | {line_no} | `{fn_name}` |")
    lines.append("")
    MAPPING_MD.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {MAPPING_MD.relative_to(ROOT)} ({len(mapping)} Java symbols mapped)")

    if args.list_unmapped:
        mfile = ROOT / "reference" / "server_mappings.txt"
        if mfile.exists():
            mapped_classes = {k.split("#", 1)[0] for k in mapping}
            unmapped_classes: list[str] = []
            for raw in mfile.read_text(encoding="utf-8", errors="ignore").splitlines():
                if not raw.startswith(" ") and not raw.startswith("#") and " -> " in raw:
                    cls = raw.split(" -> ", 1)[0].strip()
                    if cls.startswith("net.minecraft.") and cls not in mapped_classes:
                        unmapped_classes.append(cls)
            print(f"Unmapped net.minecraft.* classes in server_mappings.txt: {len(unmapped_classes)}")
            for cls in unmapped_classes[:25]:
                print(f"  - {cls}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
