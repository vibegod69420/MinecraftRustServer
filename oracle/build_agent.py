#!/usr/bin/env python3
"""Compile oracle-agent.jar (ASM 9.7.1 + OracleAgent + OracleRuntime)."""

from __future__ import annotations

import shutil
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASM_JAR = ROOT / "reference" / "tools" / "asm-9.7.1.jar"
SRC_DIR = ROOT / "oracle" / "agent" / "src"
BUILD_DIR = ROOT / "oracle" / "build"
CLASSES_DIR = BUILD_DIR / "agent_classes"
AGENT_JAR = BUILD_DIR / "oracle-agent.jar"


def build_agent() -> Path:
    if not ASM_JAR.exists():
        raise FileNotFoundError(f"Missing ASM jar at {ASM_JAR}; run scripts/fetch_reference.py or download ASM 9.7.1")

    if CLASSES_DIR.exists():
        shutil.rmtree(CLASSES_DIR)
    CLASSES_DIR.mkdir(parents=True, exist_ok=True)

    # Extract ASM classes so oracle-agent.jar is completely self-contained
    with zipfile.ZipFile(ASM_JAR, "r") as zf:
        for item in zf.infolist():
            if item.filename.startswith("org/objectweb/asm/") and not item.is_dir():
                out = CLASSES_DIR / item.filename
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_bytes(zf.read(item.filename))

    java_files = [str(p) for p in SRC_DIR.rglob("*.java")]
    subprocess.run(
        [
            "javac",
            "--release",
            "21",
            "-cp",
            str(ASM_JAR),
            "-d",
            str(CLASSES_DIR),
            *java_files,
        ],
        check=True,
    )

    manifest = BUILD_DIR / "MANIFEST.MF"
    manifest.write_text(
        "Manifest-Version: 1.0\n"
        "Premain-Class: mc.oracle.OracleAgent\n"
        "Can-Redefine-Classes: false\n"
        "Can-Retransform-Classes: false\n",
        encoding="utf-8",
    )

    subprocess.run(
        ["jar", "cfm", str(AGENT_JAR), str(manifest), "-C", str(CLASSES_DIR), "."],
        check=True,
    )
    print(f"[build_agent] Built {AGENT_JAR.relative_to(ROOT)} ({AGENT_JAR.stat().st_size} bytes)")
    return AGENT_JAR


if __name__ == "__main__":
    build_agent()
