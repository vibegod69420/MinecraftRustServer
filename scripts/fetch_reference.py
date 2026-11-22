#!/usr/bin/env python3
"""
Fetch official Minecraft 1.21.11 server artifacts, extract bundler contents,
run vanilla data generator, optionally remap via official Mojang mappings,
and decompile core packages with Vineflower.

All outputs live under `reference/`, which is gitignored and MUST remain private.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REF = ROOT / "reference"
MANIFEST_V2_URL = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
TARGET_VERSION = "1.21.11"
VINEFLOWER_URL = "https://github.com/Vineflower/vineflower/releases/download/1.12.0/vineflower-1.12.0.jar"
SPECIALSOURCE_URL = "https://repo1.maven.org/maven2/net/md-5/SpecialSource/1.11.4/SpecialSource-1.11.4-shaded.jar"

CORE_PREFIXES = (
    "net/minecraft/util/",
    "net/minecraft/nbt/",
    "net/minecraft/core/",
    "net/minecraft/network/",
    "net/minecraft/world/level/levelgen/",
    "net/minecraft/world/level/chunk/",
    "net/minecraft/world/level/storage/",
    "net/minecraft/server/TickTask",
    "net/minecraft/server/MinecraftServer",
    "net/minecraft/server/ServerTickRateManager",
)


def sha1_file(path: Path) -> str:
    h = hashlib.sha1()
    with path.open("rb") as f:
        while chunk := f.read(1 << 20):
            h.update(chunk)
    return h.hexdigest()


def download(url: str, dest: Path, expected_sha1: str | None = None) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        if expected_sha1 is None or sha1_file(dest).lower() == expected_sha1.lower():
            print(f"[skip] {dest.relative_to(ROOT)} already present")
            return
        print(f"[redownload] {dest.relative_to(ROOT)} checksum mismatch")
    print(f"[download] {url} -> {dest.relative_to(ROOT)}")
    req = urllib.request.Request(url, headers={"User-Agent": "mc-rust-reference-fetcher/1.0"})
    with urllib.request.urlopen(req, timeout=120) as resp, dest.open("wb") as out:
        shutil.copyfileobj(resp, out)
    if expected_sha1 is not None:
        got = sha1_file(dest)
        if got.lower() != expected_sha1.lower():
            raise RuntimeError(f"SHA1 mismatch for {dest}: expected {expected_sha1}, got {got}")


def fetch_version_json() -> dict:
    vpath = REF / "version.json"
    REF.mkdir(parents=True, exist_ok=True)
    if not vpath.exists():
        print(f"[fetch] {MANIFEST_V2_URL}")
        with urllib.request.urlopen(MANIFEST_V2_URL, timeout=30) as resp:
            manifest = json.loads(resp.read().decode("utf-8"))
        entry = next((v for v in manifest["versions"] if v["id"] == TARGET_VERSION), None)
        if not entry:
            raise RuntimeError(f"Version {TARGET_VERSION} not found in manifest")
        download(entry["url"], vpath, entry.get("sha1"))
    return json.loads(vpath.read_text(encoding="utf-8"))


def extract_bundler(server_jar: Path) -> tuple[Path, Path, bool]:
    lib_dir = REF / "libraries"
    lib_dir.mkdir(parents=True, exist_ok=True)
    inner_jar = REF / f"server-{TARGET_VERSION}-inner.jar"

    with zipfile.ZipFile(server_jar, "r") as zf:
        names = set(zf.namelist())
        if "META-INF/versions.list" in names:
            vlist = zf.read("META-INF/versions.list").decode("utf-8").strip().splitlines()
            print(f"[bundler] versions.list: {vlist}")
            for line in vlist:
                parts = line.split("\t")
                if len(parts) >= 3:
                    entry_path = "META-INF/versions/" + parts[2]
                    if entry_path in names:
                        with zf.open(entry_path) as src, inner_jar.open("wb") as dst:
                            shutil.copyfileobj(src, dst)
        else:
            print("[bundler] No META-INF/versions.list; using server.jar directly")
            shutil.copy2(server_jar, inner_jar)

        if "META-INF/libraries.list" in names:
            llist = zf.read("META-INF/libraries.list").decode("utf-8").strip().splitlines()
            print(f"[bundler] extracting {len(llist)} libraries...")
            for line in llist:
                parts = line.split("\t")
                if len(parts) >= 3:
                    entry_path = "META-INF/libraries/" + parts[2]
                    out_lib = lib_dir / parts[2]
                    if entry_path in names and not out_lib.exists():
                        out_lib.parent.mkdir(parents=True, exist_ok=True)
                        with zf.open(entry_path) as src, out_lib.open("wb") as dst:
                            shutil.copyfileobj(src, dst)

    with zipfile.ZipFile(inner_jar, "r") as zf:
        inner_names = zf.namelist()
        is_deobf = any(n.startswith("net/minecraft/util/Mth.class") for n in inner_names)
        print(f"[bundler] inner jar entries={len(inner_names)}, already_deobfuscated={is_deobf}")

    return inner_jar, lib_dir, is_deobf


def maybe_remap(inner_jar: Path, mappings_path: Path | None, is_deobf: bool) -> Path:
    if is_deobf:
        print("[remap] Inner jar already has Mojang names; skipping remap")
        return inner_jar
    if not mappings_path or not mappings_path.exists():
        raise RuntimeError("Inner jar appears obfuscated, but server_mappings.txt is missing!")
    mapped_jar = REF / f"server-{TARGET_VERSION}-mapped.jar"
    if mapped_jar.exists():
        print(f"[skip] {mapped_jar.relative_to(ROOT)} already present")
        return mapped_jar
    ss_jar = REF / "tools" / "SpecialSource.jar"
    download(SPECIALSOURCE_URL, ss_jar)
    print("[remap] Remapping server jar with Mojang official mappings via SpecialSource...")
    cmd = [
        "java",
        "-jar",
        str(ss_jar),
        "--in-jar",
        str(inner_jar),
        "--out-jar",
        str(mapped_jar),
        "--srg-in",
        str(mappings_path),
    ]
    subprocess.run(cmd, check=True)
    # Strip Mojang signature files from remapped jar so classpath verification never fails
    tmp_jar = mapped_jar.with_suffix(".tmp.jar")
    with zipfile.ZipFile(mapped_jar, "r") as zin, zipfile.ZipFile(tmp_jar, "w") as zout:
        for item in zin.infolist():
            if item.filename.startswith("META-INF/MOJANGCS."):
                continue
            zout.writestr(item, zin.read(item.filename))
    tmp_jar.replace(mapped_jar)
    return mapped_jar


def run_data_generator(server_jar: Path) -> None:
    gen_dir = REF / "generated"
    reports_dir = gen_dir / "reports"
    if (reports_dir / "packets.json").exists() and (reports_dir / "blocks.json").exists():
        print("[skip] reference/generated/reports already present")
        return
    work_dir = REF / "datagen_work"
    work_dir.mkdir(parents=True, exist_ok=True)
    gen_dir.mkdir(parents=True, exist_ok=True)
    cmd = [
        "java",
        f"-DbundlerRepoDir={REF / 'bundler_repo'}",
        "-DbundlerMainClass=net.minecraft.data.Main",
        "-jar",
        str(server_jar),
        "--all",
        "--output",
        str(gen_dir),
    ]
    print(f"[datagen] Running: {' '.join(cmd)}")
    subprocess.run(cmd, cwd=work_dir, check=True)


def decompile_sources(jar_path: Path, lib_dir: Path, decompile_all: bool) -> None:
    vf_jar = REF / "tools" / "vineflower.jar"
    download(VINEFLOWER_URL, vf_jar)
    out_dir = REF / "decompiled"
    marker = out_dir / (".all.done" if decompile_all else ".core.done")
    if marker.exists():
        print(f"[skip] {out_dir.relative_to(ROOT)} already decompiled ({marker.name})")
        return
    out_dir.mkdir(parents=True, exist_ok=True)

    target_jar = jar_path
    if not decompile_all:
        subset_jar = REF / "server-core-subset.jar"
        print(f"[decompile] Creating core package subset jar -> {subset_jar.relative_to(ROOT)}")
        with zipfile.ZipFile(jar_path, "r") as zin, zipfile.ZipFile(subset_jar, "w") as zout:
            for item in zin.infolist():
                if any(item.filename.startswith(p) for p in CORE_PREFIXES):
                    zout.writestr(item, zin.read(item.filename))
        target_jar = subset_jar

    cmd = ["java", "-Xmx2G", "-jar", str(vf_jar), "--folder"]
    for lib in sorted(lib_dir.rglob("*.jar")):
        cmd.append(f"-e={lib}")
    cmd.extend([str(target_jar), str(out_dir)])
    print(f"[decompile] Running Vineflower on {target_jar.name}...")
    subprocess.run(cmd, check=True)
    marker.write_text("ok\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--decompile-all", action="store_true", help="Decompile full server jar instead of core packages")
    parser.add_argument("--skip-decompile", action="store_true", help="Skip Vineflower decompilation")
    args = parser.parse_args()

    vjson = fetch_version_json()
    downloads = vjson.get("downloads", {})
    srv_info = downloads.get("server")
    if not srv_info:
        raise RuntimeError("No server download in version.json")

    server_jar = REF / "server.jar"
    download(srv_info["url"], server_jar, srv_info.get("sha1"))

    mappings_path: Path | None = None
    map_info = downloads.get("server_mappings")
    if map_info:
        mappings_path = REF / "server_mappings.txt"
        download(map_info["url"], mappings_path, map_info.get("sha1"))
    else:
        print("[info] No server_mappings entry in version.json (release may be unobfuscated)")

    inner_jar, lib_dir, is_deobf = extract_bundler(server_jar)
    mapped_jar = maybe_remap(inner_jar, mappings_path, is_deobf)
    run_data_generator(server_jar)

    if not args.skip_decompile:
        decompile_sources(mapped_jar, lib_dir, args.decompile_all)

    print("[done] Reference setup complete.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
