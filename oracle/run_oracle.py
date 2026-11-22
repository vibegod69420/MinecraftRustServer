#!/usr/bin/env python3
"""
Deterministic Oracle Driver & Self-Test (Step 0).

Runs the 1.21.11 vanilla server with:
  - oracle-agent.jar (deterministic unique seeds, UUIDs, unseeded Random, startup freeze, RNG trace)
  - -Dmax.bg.threads=1
  - -XX:+UnlockExperimentalVMOptions -XX:hashCode=2
Then parses level.dat and Anvil .mca chunk NBT structurally and computes a SHA-256 Merkle hash.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import shutil
import struct
import subprocess
import sys
import threading
import time
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "oracle"))
import build_agent  # noqa: E402

MAPPED_JAR = ROOT / "reference" / "server-1.21.11-mapped.jar"
LIB_DIR = ROOT / "reference" / "libraries"
RUN_ROOT = ROOT / "oracle" / "run"


# --- Minimal Structural NBT Parser & Canonical Serializer (Method 3 & 5) ---

def _read_u8(buf: io.BytesIO) -> int:
    b = buf.read(1)
    if len(b) != 1:
        raise EOFError("Unexpected EOF in NBT")
    return b[0]


def _read_fmt(buf: io.BytesIO, fmt: str):
    sz = struct.calcsize(fmt)
    b = buf.read(sz)
    if len(b) != sz:
        raise EOFError("Unexpected EOF in NBT")
    return struct.unpack(fmt, b)[0]


def _read_nbt_string(buf: io.BytesIO) -> str:
    n = _read_fmt(buf, ">H")
    raw = buf.read(n)
    if len(raw) != n:
        raise EOFError("Unexpected EOF in NBT string")
    return raw.decode("utf-8", errors="surrogatepass")


def _read_tag_payload(buf: io.BytesIO, tag_id: int):
    if tag_id == 0:
        return None
    if tag_id == 1:
        return ("byte", _read_fmt(buf, ">b"))
    if tag_id == 2:
        return ("short", _read_fmt(buf, ">h"))
    if tag_id == 3:
        return ("int", _read_fmt(buf, ">i"))
    if tag_id == 4:
        return ("long", _read_fmt(buf, ">q"))
    if tag_id == 5:
        return ("float", _read_fmt(buf, ">I"))  # raw IEEE-754 bits
    if tag_id == 6:
        return ("double", _read_fmt(buf, ">Q"))  # raw IEEE-754 bits
    if tag_id == 7:
        n = _read_fmt(buf, ">i")
        return ("bytes", buf.read(n).hex())
    if tag_id == 8:
        return ("string", _read_nbt_string(buf))
    if tag_id == 9:
        elem_id = _read_u8(buf)
        n = _read_fmt(buf, ">i")
        return ("list", elem_id, [_read_tag_payload(buf, elem_id) for _ in range(n)])
    if tag_id == 10:
        entries = {}
        while True:
            tid = _read_u8(buf)
            if tid == 0:
                break
            name = _read_nbt_string(buf)
            entries[name] = _read_tag_payload(buf, tid)
        return ("compound", entries)
    if tag_id == 11:
        n = _read_fmt(buf, ">i")
        return ("int_array", list(struct.unpack(f">{n}i", buf.read(4 * n))))
    if tag_id == 12:
        n = _read_fmt(buf, ">i")
        return ("long_array", list(struct.unpack(f">{n}q", buf.read(8 * n))))
    raise ValueError(f"Unknown NBT tag id: {tag_id}")


def parse_nbt_bytes(raw: bytes) -> tuple[str, object]:
    buf = io.BytesIO(raw)
    root_id = _read_u8(buf)
    if root_id == 0:
        return ("", None)
    name = _read_nbt_string(buf)
    return (name, _read_tag_payload(buf, root_id))


def canonicalize_nbt(node, ignore_keys: set[str] | None = None) -> object:
    if isinstance(node, tuple) and node and node[0] == "compound":
        items = {}
        for k in sorted(node[1].keys()):
            if ignore_keys and k in ignore_keys:
                continue
            items[k] = canonicalize_nbt(node[1][k], ignore_keys)
        return ["compound", items]
    if isinstance(node, tuple) and node and node[0] == "list":
        return ["list", node[1], [canonicalize_nbt(x, ignore_keys) for x in node[2]]]
    if isinstance(node, tuple):
        return list(node)
    return node


def hash_canonical_nbt(node, ignore_keys: set[str] | None = None) -> str:
    canon = canonicalize_nbt(node, ignore_keys)
    blob = json.dumps(canon, separators=(",", ":"), ensure_ascii=True).encode("utf-8")
    return hashlib.sha256(blob).hexdigest()


def parse_mca_chunks(mca_path: Path) -> dict[tuple[int, int], tuple[str, object]]:
    raw = mca_path.read_bytes()
    if len(raw) < 8192:
        return {}
    chunks: dict[tuple[int, int], tuple[str, object]] = {}
    for idx in range(1024):
        loc = struct.unpack(">I", raw[idx * 4 : idx * 4 + 4])[0]
        if loc == 0:
            continue
        sector_off = (loc >> 8) * 4096
        if sector_off + 5 > len(raw):
            continue
        length, comp_type = struct.unpack(">IB", raw[sector_off : sector_off + 5])
        payload = raw[sector_off + 5 : sector_off + 4 + length]
        if comp_type == 1:
            nbt_bytes = gzip.decompress(payload)
        elif comp_type == 2:
            nbt_bytes = zlib.decompress(payload)
        elif comp_type == 3:
            nbt_bytes = payload
        else:
            raise ValueError(f"Unsupported MCA compression type {comp_type} in {mca_path}")
        _, tag = parse_nbt_bytes(nbt_bytes)
        cx = idx & 31
        cz = idx >> 5
        chunks[(cx, cz)] = (hash_canonical_nbt(tag), tag)
    return chunks


def compute_world_merkle(world_dir: Path) -> dict:
    # Ignore LastPlayed in level.dat (wall-clock epoch millis) while recording its raw value
    level_dat = world_dir / "level.dat"
    _, level_tag = parse_nbt_bytes(gzip.decompress(level_dat.read_bytes()))
    level_hash = hash_canonical_nbt(level_tag, ignore_keys={"LastPlayed"})

    dim_dirs = {
        "overworld": world_dir,
        "the_nether": world_dir / "DIM-1",
        "the_end": world_dir / "DIM1",
    }
    dim_hashes = {}
    chunk_hashes_all = {}

    for dim_name, ddir in sorted(dim_dirs.items()):
        sub_hashes = {}
        for folder in ("region", "entities", "poi"):
            fdir = ddir / folder
            if not fdir.exists():
                continue
            for mca in sorted(fdir.glob("*.mca")):
                chunks = parse_mca_chunks(mca)
                for (cx, cz), (chash, _) in sorted(chunks.items()):
                    key = f"{dim_name}/{folder}/{mca.name}/{cx},{cz}"
                    sub_hashes[key] = chash
                    chunk_hashes_all[key] = chash
        dim_blob = json.dumps(sub_hashes, sort_keys=True, separators=(",", ":")).encode("utf-8")
        dim_hashes[dim_name] = {
            "chunk_count": len(sub_hashes),
            "hash": hashlib.sha256(dim_blob).hexdigest(),
        }

    world_payload = json.dumps(
        {"level_dat": level_hash, "dimensions": dim_hashes},
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return {
        "world_merkle_hash": hashlib.sha256(world_payload).hexdigest(),
        "level_dat_hash": level_hash,
        "dimensions": dim_hashes,
        "chunk_hashes": chunk_hashes_all,
    }


def run_scenario(scenario_path: Path, run_id: str, trace_rng: bool = True) -> dict:
    agent_jar = build_agent.build_agent()
    scenario = json.loads(scenario_path.read_text(encoding="utf-8"))

    run_dir = RUN_ROOT / run_id
    if run_dir.exists():
        shutil.rmtree(run_dir)
    run_dir.mkdir(parents=True, exist_ok=True)

    (run_dir / "eula.txt").write_text("eula=true\n", encoding="utf-8")
    props = [
        "online-mode=false",
        f"level-seed={scenario.get('seed', 12345)}",
        f"level-type={scenario.get('level_type', 'minecraft:flat')}",
        "server-port=0",
        "spawn-protection=0",
        "max-tick-time=-1",
        "pause-when-empty-seconds=0",
        "view-distance=4",
        "simulation-distance=4",
        "sync-chunk-writes=true",
        "enable-status=false",
    ]
    (run_dir / "server.properties").write_text("\n".join(props) + "\n", encoding="utf-8")

    trace_path = run_dir / "rng_trace.tsv"
    jars = [str(agent_jar), str(MAPPED_JAR)] + [str(p) for p in sorted(LIB_DIR.rglob("*.jar"))]
    sep = ";" if sys.platform == "win32" else ":"

    cmd = [
        "java",
        "-Xmx1536M",
        "-XX:+UnlockExperimentalVMOptions",
        "-XX:hashCode=2",
        "-Dmax.bg.threads=1",
        f"-javaagent:{agent_jar}",
        "-Dmc.oracle.deterministic=true",
        "-Dmc.oracle.startFrozen=true",
        f"-Dmc.oracle.seed={scenario.get('seed', 12345)}",
    ]
    if trace_rng:
        cmd.append(f"-Dmc.oracle.rng_trace={trace_path}")
    cmd.extend([
        "-cp",
        sep.join(jars),
        "net.minecraft.server.Main",
        "--nogui",
        "--port",
        "0",
    ])

    proc = subprocess.Popen(
        cmd,
        cwd=run_dir,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
        bufsize=1,
    )

    lines: list[str] = []
    ready_ev = threading.Event()
    step_done_ev = threading.Event()
    saved_ev = threading.Event()
    step_target = int(scenario.get("ticks", 20))

    def reader():
        assert proc.stdout is not None
        for line in proc.stdout:
            lines.append(line.rstrip("\r\n"))
            if "Done (" in line:
                ready_ev.set()
            if "Sprint completed with" in line:
                step_done_ev.set()
            if "Saved the game" in line:
                saved_ev.set()

    t = threading.Thread(target=reader, daemon=True)
    t.start()

    def wait_event_or_exit(ev: threading.Event, timeout_s: float, label: str):
        deadline = time.monotonic() + timeout_s
        while not ev.is_set():
            if proc.poll() is not None:
                raise RuntimeError(f"Oracle exited early (rc={proc.returncode}) while waiting for {label}:\n" + "\n".join(lines[-40:]))
            if time.monotonic() >= deadline:
                raise RuntimeError(f"Oracle timed out waiting for {label}:\n" + "\n".join(lines[-40:]))
            ev.wait(timeout=0.2)

    try:
        wait_event_or_exit(ready_ev, 180.0, "server ready")

        def send(c: str):
            assert proc.stdin is not None
            proc.stdin.write(c + "\n")
            proc.stdin.flush()

        for sc in scenario.get("setup_commands", []):
            send(sc)
            time.sleep(0.05)

        if step_target > 0:
            send(f"tick sprint {step_target}")
            wait_event_or_exit(step_done_ev, 120.0, "tick sprint completion")

        send("save-all flush")
        if not saved_ev.wait(timeout=60):
            time.sleep(1.0)
        send("stop")
        proc.wait(timeout=60)
    finally:
        if proc.poll() is None:
            proc.kill()
        t.join(timeout=5)

    if proc.returncode != 0:
        raise RuntimeError(f"Oracle exited with code {proc.returncode}:\n" + "\n".join(lines[-40:]))

    world_dir = run_dir / "world"
    merkle = compute_world_merkle(world_dir)
    trace_hash = None
    trace_lines = 0
    if trace_rng and trace_path.exists():
        h = hashlib.sha256()
        with trace_path.open("rb") as f:
            for chunk in iter(lambda: f.read(1 << 20), b""):
                h.update(chunk)
        trace_hash = h.hexdigest()
        with trace_path.open("r", encoding="utf-8") as f:
            trace_lines = sum(1 for _ in f)

    return {
        "run_id": run_id,
        "merkle": merkle,
        "rng_trace_sha256": trace_hash,
        "rng_trace_lines": trace_lines,
        "log_tail": lines[-15:],
    }


def self_test(scenario_path: Path) -> int:
    print(f"[self-test] Running scenario {scenario_path.relative_to(ROOT)} twice...")
    r1 = run_scenario(scenario_path, "self_test_run1", trace_rng=True)
    r2 = run_scenario(scenario_path, "self_test_run2", trace_rng=True)

    print(f"  Run 1 world Merkle: {r1['merkle']['world_merkle_hash']} | RNG trace: {r1['rng_trace_sha256']} ({r1['rng_trace_lines']} lines)")
    print(f"  Run 2 world Merkle: {r2['merkle']['world_merkle_hash']} | RNG trace: {r2['rng_trace_sha256']} ({r2['rng_trace_lines']} lines)")

    ok = True
    if r1["merkle"]["world_merkle_hash"] != r2["merkle"]["world_merkle_hash"]:
        ok = False
        print("  [DIFF] World Merkle hash mismatch!")
        if r1["merkle"]["level_dat_hash"] != r2["merkle"]["level_dat_hash"]:
            print("    - level.dat differs!")
        c1 = r1["merkle"]["chunk_hashes"]
        c2 = r2["merkle"]["chunk_hashes"]
        for k in sorted(set(c1) | set(c2)):
            if c1.get(k) != c2.get(k):
                print(f"    - chunk differs: {k} ({c1.get(k)} != {c2.get(k)})")

    if r1["rng_trace_sha256"] != r2["rng_trace_sha256"]:
        ok = False
        print("  [DIFF] RNG trace differs! Finding first differing line...")
        p1 = RUN_ROOT / "self_test_run1" / "rng_trace.tsv"
        p2 = RUN_ROOT / "self_test_run2" / "rng_trace.tsv"
        with p1.open("r", encoding="utf-8") as f1, p2.open("r", encoding="utf-8") as f2:
            for lineno, (a, b) in enumerate(zip(f1, f2), 1):
                if a != b:
                    print(f"    First diff at RNG line {lineno}:\n      R1: {a.strip()}\n      R2: {b.strip()}")
                    break

    if ok:
        print("[self-test] PASS: Both oracle runs produced BIT-IDENTICAL world state and RNG traces!")
        return 0
    return 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--scenario", default=str(ROOT / "scenarios" / "self_test_flat.json"))
    args = parser.parse_args()

    if args.self_test:
        return self_test(Path(args.scenario))
    res = run_scenario(Path(args.scenario), "manual_run", trace_rng=True)
    print(json.dumps({k: v for k, v in res.items() if k != "merkle"}, indent=2))
    print("world_merkle_hash:", res["merkle"]["world_merkle_hash"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
