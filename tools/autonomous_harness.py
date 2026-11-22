#!/usr/bin/env python3
"""
24/7 Autonomous Phase-A Porting Harness for Minecraft 1.21.11 -> Rust.

Uses DeepSeek V4.1 Flash (`api.deepai.org/deepai_code`, `reasoning_effort: max`, 320K context)
to port server classes from `reference/decompiled/` into `crates/<crate>/src/ported/<module>.rs`
in dependency order, verifying every module with `cargo test --workspace` and publishing
atomic `docs/worker.json` + `docs/progress.json` updates to GitHub.
"""

from __future__ import annotations

import contextlib
import json
import os
import re
import subprocess
import sys
import threading
import time
from collections import deque
from concurrent.futures import FIRST_COMPLETED, ThreadPoolExecutor, wait
from pathlib import Path

try:
    import fcntl
except ImportError:  # Windows
    fcntl = None  # type: ignore[assignment]

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import deepai_backend  # noqa: E402
import update_progress  # noqa: E402

DECOMPILED_DIR = ROOT / "reference" / "decompiled"
CRATES_DIR = ROOT / "crates"
DATA_DIR = ROOT / "data"
LEDGER_PATH = DATA_DIR / "work_ledger.jsonl"
SKIP_PATH = DATA_DIR / "skipped.json"
WORKERS_FILE = DATA_DIR / "workers.txt"
WORKER_STATUS = ROOT / "docs" / "worker.json"
COMPLETION_FLAG = ROOT / "COMPLETED_100_PERCENT.flag"

SKIP_TTL_SECONDS = 12 * 3600
SKIP_AFTER_FAILURES = 2
WATCHDOG_STALE_SECONDS = int(os.environ.get("MC_RUST_WATCHDOG_STALE", "3600"))
MAX_OUTPUT_TOKENS = int(os.environ.get("MC_RUST_MAX_TOKENS", "65536"))
LOG_PATH = Path(os.environ.get("MC_RUST_LOG_PATH", "/var/log/mc-rust/harness.log"))

LAST_ACTIVITY = time.time()
WRITE_LOCK = threading.Lock()
REPO_LOCK_PATH = Path("/tmp/mc-rust-repo.lock")
_STATUS_LOCK = threading.RLock()


@contextlib.contextmanager
def repo_write_lock():
    """Serialize module writes, cargo tests, rollbacks, and git commits across threads and publish.sh."""
    with WRITE_LOCK:
        if fcntl is None:
            yield
            return
        with open(REPO_LOCK_PATH, "w", encoding="utf-8") as lock_fh:
            fcntl.flock(lock_fh.fileno(), fcntl.LOCK_EX)
            try:
                yield
            finally:
                fcntl.flock(lock_fh.fileno(), fcntl.LOCK_UN)
_ACTIVE_CLASSES: dict[str, dict] = {}
_STATUS: dict = {
    "state": "starting",
    "class_name": None,
    "crate": None,
    "attempt": 0,
    "verified_count": 0,
    "workers": 0,
    "active_classes": [],
}

LOW_PRIORITY_PREFIXES = (
    "net/minecraft/util/profiling/",
    "net/minecraft/util/debugchart/",
    "net/minecraft/util/parsing/",
    "net/minecraft/util/datafix/",
    "net/minecraft/util/eventlog/",
    "net/minecraft/util/monitoring/",
    "net/minecraft/gametest/",
)

# Dependency-ordered package tiers matching PLAN.md
PACKAGE_PRIORITY = [
    ("net/minecraft/nbt/", "mc-nbt"),
    ("net/minecraft/util/random/", "mc-math"),
    ("net/minecraft/util/valueproviders/", "mc-math"),
    ("net/minecraft/world/level/levelgen/synth/", "mc-math"),
    ("net/minecraft/core/", "mc-math"),
    ("net/minecraft/network/", "mc-protocol"),
    ("net/minecraft/world/level/levelgen/", "mc-worldgen"),
    ("net/minecraft/world/level/chunk/", "mc-worldgen"),
    ("net/minecraft/world/level/storage/", "mc-nbt"),
    ("net/minecraft/util/", "mc-math"),
    ("net/minecraft/world/level/block/", "mc-worldgen"),
    ("net/minecraft/world/level/material/", "mc-worldgen"),
    ("net/minecraft/world/level/", "mc-worldgen"),
    ("net/minecraft/world/entity/", "mc-worldgen"),
    ("net/minecraft/server/", "mc-protocol"),
]

LOW_PRIORITY_TIER = len(PACKAGE_PRIORITY) + 10


def log(msg: str) -> None:
    global LAST_ACTIVITY
    LAST_ACTIVITY = time.time()
    ts = time.strftime("%Y-%m-%d %H:%M:%S")
    print(f"[{ts}] {msg}", flush=True)


def publish_status(**fields) -> None:
    with _STATUS_LOCK:
        _STATUS.update(fields)
        _STATUS["active_classes"] = list(_ACTIVE_CLASSES.values())
        _STATUS["ts"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        try:
            WORKER_STATUS.parent.mkdir(parents=True, exist_ok=True)
            tmp = WORKER_STATUS.with_suffix(".tmp")
            tmp.write_text(json.dumps(_STATUS), encoding="utf-8")
            tmp.replace(WORKER_STATUS)
        except OSError:
            pass


def set_worker_activity(fqcn: str, crate: str, attempt: int, state: str) -> None:
    with _STATUS_LOCK:
        _ACTIVE_CLASSES[fqcn] = {
            "class_name": fqcn,
            "crate": crate,
            "attempt": attempt,
            "state": state,
        }
        publish_status(state=state, class_name=fqcn, crate=crate, attempt=attempt)


def clear_worker_activity(fqcn: str) -> None:
    with _STATUS_LOCK:
        _ACTIVE_CLASSES.pop(fqcn, None)
        if _ACTIVE_CLASSES:
            latest = next(reversed(_ACTIVE_CLASSES.values()))
            publish_status(**latest)
        else:
            publish_status(state="idle", class_name=None, crate=None, attempt=0)


def rotate_log_if_needed(max_bytes: int = 64 * 1024 * 1024, keep: int = 8 * 1024 * 1024) -> None:
    try:
        if not LOG_PATH.exists() or LOG_PATH.stat().st_size <= max_bytes:
            return
        with open(LOG_PATH, "rb") as fh:
            fh.seek(-keep, os.SEEK_END)
            tail = fh.read()
        with open(LOG_PATH, "wb") as fh:
            fh.write(b"[log rotated]\n")
            fh.write(tail)
    except OSError:
        pass


def watchdog_loop() -> None:
    last_rot = time.time()
    while True:
        time.sleep(60)
        if time.time() - last_rot > 3600:
            rotate_log_if_needed()
            last_rot = time.time()
        idle = time.time() - LAST_ACTIVITY
        if idle > WATCHDOG_STALE_SECONDS:
            log(f"WATCHDOG: no activity for {int(idle)}s; exiting for systemd restart")
            sys.stdout.flush()
            os._exit(1)


def load_skips() -> dict:
    try:
        raw = json.loads(SKIP_PATH.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    now = time.time()
    return {k: v for k, v in raw.items() if isinstance(v, list) and now - v[1] < SKIP_TTL_SECONDS}


def save_skips(skips: dict) -> None:
    try:
        SKIP_PATH.parent.mkdir(parents=True, exist_ok=True)
        SKIP_PATH.write_text(json.dumps(skips), encoding="utf-8")
    except OSError:
        pass


def record_work(fqcn: str, crate: str, verified: bool, seconds: float, note: str = "") -> None:
    try:
        LEDGER_PATH.parent.mkdir(parents=True, exist_ok=True)
        with LEDGER_PATH.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps({
                "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "class_name": fqcn,
                "crate": crate,
                "verified": verified,
                "seconds": round(seconds, 1),
                "note": note[:200],
            }) + "\n")
    except OSError:
        pass


def snake_case(name: str) -> str:
    s1 = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    s2 = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s1).lower()
    s2 = re.sub(r"[^a-z0-9_]", "_", s2).strip("_")
    if s2 in {"type", "match", "loop", "impl", "mod", "struct", "enum", "fn", "ref", "use", "async", "await"}:
        s2 += "_mc"
    return s2 or "ported_item"


def route_java_file(rel_posix: str) -> tuple[int, str, str]:
    """Return (priority_tier, target_crate, module_name) for a decompiled .java file."""
    fqcn = rel_posix[:-5].replace("/", ".")
    short = fqcn.rsplit(".", 1)[-1]
    mod = snake_case(short)
    matched_tier = len(PACKAGE_PRIORITY)
    matched_crate = "mc-worldgen"
    for tier, (prefix, crate) in enumerate(PACKAGE_PRIORITY):
        if rel_posix.startswith(prefix):
            matched_tier = tier
            matched_crate = crate
            break
    if rel_posix.startswith(LOW_PRIORITY_PREFIXES):
        return (LOW_PRIORITY_TIER, matched_crate, mod)
    return (matched_tier, matched_crate, mod)


def _interleave_by_crate(items: list[dict]) -> list[dict]:
    crate_order = ["mc-nbt", "mc-math", "mc-protocol", "mc-worldgen"]
    buckets: dict[str, deque[dict]] = {}
    for item in items:
        buckets.setdefault(item["crate"], deque()).append(item)
    ordered_keys = [c for c in crate_order if c in buckets] + [c for c in buckets if c not in crate_order]
    out: list[dict] = []
    while any(buckets.values()):
        for crate in ordered_keys:
            q = buckets.get(crate)
            if q:
                out.append(q.popleft())
    return out


def list_candidate_classes() -> list[dict]:
    if not DECOMPILED_DIR.exists():
        return []
    mapped, _ = update_progress.scan_annotations()
    mapped_classes = {k.split("#", 1)[0] for k in mapped}

    candidates = []
    for jfile in sorted(DECOMPILED_DIR.rglob("*.java")):
        rel = jfile.relative_to(DECOMPILED_DIR).as_posix()
        if not rel.startswith("net/minecraft/"):
            continue
        if rel.startswith("net/minecraft/client/") or rel.endswith("package-info.java"):
            continue
        if "$" in rel:
            continue
        fqcn = rel[:-5].replace("/", ".")
        if fqcn in mapped_classes:
            continue
        try:
            sz = jfile.stat().st_size
        except OSError:
            continue
        # Skip huge classes > 64KB in early Phase A sweeps; tackle manageable building blocks first
        if sz < 120 or sz > 64000:
            continue
        tier, crate, mod = route_java_file(rel)
        candidates.append({
            "tier": tier,
            "size": sz,
            "fqcn": fqcn,
            "rel": rel,
            "path": jfile,
            "crate": crate,
            "mod": mod,
        })

    # Prefer dependency tier, then smaller self-contained classes first,
    # keeping low-priority utilities after core classes and interleaving round-robin by crate.
    candidates.sort(key=lambda c: (c["tier"], c["size"], c["fqcn"]))
    core_candidates = [c for c in candidates if c["tier"] < LOW_PRIORITY_TIER]
    low_candidates = [c for c in candidates if c["tier"] >= LOW_PRIORITY_TIER]
    return _interleave_by_crate(core_candidates) + _interleave_by_crate(low_candidates)


def extract_rust_block(text: str) -> str:
    m = re.search(r"```(?:rust|rs)\s*\n(.*?)```", text, flags=re.DOTALL | re.IGNORECASE)
    if m:
        return m.group(1).strip()
    m2 = re.search(r"```\s*\n(.*?)```", text, flags=re.DOTALL)
    if m2:
        return m2.group(1).strip()
    return ""


def parse_batch_rust_blocks(text: str, expected_mods: list[str]) -> dict[str, str]:
    """Extract per-module Rust code blocks from a batched LLM response."""
    expected_set = set(expected_mods)
    if not expected_mods:
        return {}
    if len(expected_mods) == 1:
        single = extract_rust_block(text)
        return {expected_mods[0]: single} if single else {}

    file_marker_re = re.compile(
        r"(?mi)^(?://\s*|#+\s*)?(?:===+\s*)?(?:FILE:\s*|[^\s]*?/ported/)([A-Za-z0-9_]+?)(?:\.rs)?(?:\s*===+)?\s*$"
    )
    inline_split_re = re.compile(
        r"(?mi)^(?://\s*)?===+\s*FILE:\s*([A-Za-z0-9_]+?)(?:\.rs)?\s*===+\s*$"
    )
    fence_re = re.compile(r"```(?:rust|rs)?\s*\n(.*?)```", flags=re.DOTALL | re.IGNORECASE)

    out: dict[str, str] = {}
    matches = list(fence_re.finditer(text))
    if not matches:
        return out

    # Handle case where LLM put all files inside a single big fenced block with inline FILE markers
    if len(matches) == 1:
        body = matches[0].group(1)
        inline_hits = [m for m in inline_split_re.finditer(body) if m.group(1) in expected_set]
        if len(inline_hits) >= 2:
            for idx, hit in enumerate(inline_hits):
                mod_name = hit.group(1)
                c_start = hit.end()
                c_end = inline_hits[idx + 1].start() if idx + 1 < len(inline_hits) else len(body)
                chunk = body[c_start:c_end].strip()
                if chunk and mod_name not in out:
                    out[mod_name] = chunk
            return out

    prev_end = 0
    for m in matches:
        pre_window = text[max(prev_end, m.start() - 600) : m.start()]
        body = m.group(1).strip()
        prev_end = m.end()
        if not body:
            continue

        body_lines = body.splitlines()
        head = "\n".join(body_lines[:3])

        matched_mod = None
        for region in (head, pre_window):
            # Prefer explicit FILE: / ported/ markers first, scan lines bottom-up in pre_window
            lines = region.splitlines()
            if region is pre_window:
                lines = list(reversed(lines))
            for ln in lines:
                s = ln.strip()
                if not s:
                    continue
                hm = file_marker_re.match(s)
                if hm and hm.group(1) in expected_set:
                    matched_mod = hm.group(1)
                    break
                for mod_name in expected_mods:
                    if re.search(rf"(?<![A-Za-z0-9_]){re.escape(mod_name)}\.rs(?![A-Za-z0-9_])", s):
                        matched_mod = mod_name
                        break
                if matched_mod:
                    break
            if matched_mod:
                break

        if matched_mod and matched_mod not in out:
            if body_lines and re.match(r"^\s*//\s*===+\s*FILE:", body_lines[0], flags=re.IGNORECASE):
                body = "\n".join(body_lines[1:]).strip()
            if body:
                out[matched_mod] = body

    return out


def build_work_units(
    candidates: list[dict],
    max_batch_size: int = 4,
    max_batch_bytes: int = 9000,
    small_threshold: int = 3500,
) -> list[list[dict]]:
    """
    Group small candidates sharing `(crate, java_package)` with distinct module names
    into batches of up to `max_batch_size` / `max_batch_bytes`, preserving priority & round-robin order.
    """
    by_pkg: dict[tuple[str, str], deque[int]] = {}
    for idx, c in enumerate(candidates):
        if c["size"] <= small_threshold:
            pkg = c["fqcn"].rsplit(".", 1)[0]
            by_pkg.setdefault((c["crate"], pkg), deque()).append(idx)

    consumed: set[int] = set()
    units: list[list[dict]] = []

    for idx, c in enumerate(candidates):
        if idx in consumed:
            continue
        consumed.add(idx)
        if c["size"] > small_threshold:
            units.append([c])
            continue

        batch = [c]
        total_bytes = c["size"]
        seen_mods = {c["mod"]}
        pkg = c["fqcn"].rsplit(".", 1)[0]
        pkg_q = by_pkg.get((c["crate"], pkg))
        if pkg_q:
            deferred: list[int] = []
            while pkg_q and len(batch) < max_batch_size:
                nxt_idx = pkg_q.popleft()
                if nxt_idx in consumed:
                    continue
                nxt = candidates[nxt_idx]
                if nxt["mod"] in seen_mods or total_bytes + nxt["size"] > max_batch_bytes:
                    deferred.append(nxt_idx)
                    continue
                consumed.add(nxt_idx)
                batch.append(nxt)
                seen_mods.add(nxt["mod"])
                total_bytes += nxt["size"]
            for d in reversed(deferred):
                pkg_q.appendleft(d)

        units.append(batch)

    return units


def _extract_crate_api_summary(crate: str, limit: int = 3000, per_file_limit: int = 650) -> str:
    """Return a compact public struct/enum/trait/impl signature summary from top-level crate sources."""
    src_dir = CRATES_DIR / crate / "src"
    if not src_dir.exists():
        return ""
    lines_out: list[str] = []
    total_chars = 0
    for rs in sorted(src_dir.glob("*.rs")):
        if rs.name == "lib.rs":
            continue
        file_sigs: list[str] = []
        in_pub_trait = False
        for raw in rs.read_text(encoding="utf-8", errors="ignore").splitlines():
            stripped = raw.strip()
            if not stripped or stripped.startswith("//"):
                continue
            indent = len(raw) - len(raw.lstrip(" "))
            if indent == 0:
                if stripped == "}":
                    in_pub_trait = False
                    continue
                if stripped.startswith("pub trait "):
                    in_pub_trait = True
                elif stripped.startswith(("impl ", "pub struct ", "pub enum ")):
                    in_pub_trait = False
                is_sig = stripped.startswith(("pub struct ", "pub enum ", "pub trait ", "pub fn ", "pub type ", "impl "))
            elif indent <= 4:
                is_sig = stripped.startswith("pub fn ") or (in_pub_trait and stripped.startswith("fn "))
            else:
                is_sig = False
            if not is_sig:
                continue
            sig = stripped.split("{", 1)[0].strip()
            if indent > 0:
                sig = "    " + sig
            if len(sig) > 120:
                sig = sig[:117] + "..."
            file_sigs.append(sig)
        if not file_sigs:
            continue
        block = f"// crate::{rs.stem}\n" + "\n".join(file_sigs)
        if len(block) > per_file_limit:
            block = block[:per_file_limit].rsplit("\n", 1)[0] + "\n// ..."
        if total_chars + len(block) > limit:
            remaining = limit - total_chars
            if remaining > 120:
                lines_out.append(block[:remaining].rsplit("\n", 1)[0] + "\n// ...")
            break
        lines_out.append(block)
        total_chars += len(block) + 2
    return "\n\n".join(lines_out)


def read_crate_context(crate: str) -> str:
    lib_rs = CRATES_DIR / crate / "src" / "lib.rs"
    cargo_toml = CRATES_DIR / crate / "Cargo.toml"
    parts = []
    if cargo_toml.exists():
        parts.append(f"### {crate}/Cargo.toml\n```toml\n{cargo_toml.read_text(encoding='utf-8')}\n```")
    if lib_rs.exists():
        parts.append(f"### {crate}/src/lib.rs\n```rust\n{lib_rs.read_text(encoding='utf-8')[:4000]}\n```")
    api_summary = _extract_crate_api_summary(crate)
    if api_summary:
        parts.append(f"### {crate} public API signatures\n```rust\n{api_summary}\n```")
    return "\n\n".join(parts)


def wire_ported_modules(crate: str, modules: list[tuple[str, str]]) -> list[tuple[Path, str | None]]:
    """
    Write one or more `crates/<crate>/src/ported/<mod_name>.rs` files atomically,
    updating `ported/mod.rs` and `lib.rs` once. Returns backups for clean rollback.
    """
    src_dir = CRATES_DIR / crate / "src"
    ported_dir = src_dir / "ported"
    ported_dir.mkdir(parents=True, exist_ok=True)
    mod_rs = ported_dir / "mod.rs"
    lib_rs = src_dir / "lib.rs"

    backups: list[tuple[Path, str | None]] = []

    def save_and_write(p: Path, content: str) -> None:
        prev = p.read_text(encoding="utf-8") if p.exists() else None
        backups.append((p, prev))
        p.write_text(content, encoding="utf-8")

    for mod_name, _ in modules:
        target_rs = ported_dir / f"{mod_name}.rs"
        if target_rs.exists():
            raise FileExistsError(f"{target_rs.relative_to(ROOT)} already exists")

    for mod_name, code in modules:
        save_and_write(ported_dir / f"{mod_name}.rs", code + "\n")

    mod_prev = mod_rs.read_text(encoding="utf-8") if mod_rs.exists() else ""
    mod_lines_to_add = [
        f"pub mod {mod_name};"
        for mod_name, _ in modules
        if f"pub mod {mod_name};" not in mod_prev
    ]
    if mod_lines_to_add:
        new_mod = (mod_prev.rstrip() + "\n" + "\n".join(mod_lines_to_add) + "\n").lstrip("\n")
        save_and_write(mod_rs, new_mod)

    lib_prev = lib_rs.read_text(encoding="utf-8")
    if "pub mod ported;" not in lib_prev:
        save_and_write(lib_rs, lib_prev.rstrip() + "\n\npub mod ported;\n")

    return backups


def wire_ported_module(crate: str, mod_name: str, code: str) -> list[tuple[Path, str | None]]:
    return wire_ported_modules(crate, [(mod_name, code)])


def rollback(backups: list[tuple[Path, str | None]]) -> None:
    for p, prev in reversed(backups):
        if prev is None:
            p.unlink(missing_ok=True)
        else:
            p.write_text(prev, encoding="utf-8")


def run_cargo_test(crate: str, mod_names: list[str] | None = None) -> tuple[bool, str]:
    env = os.environ.copy()
    env.setdefault("CARGO_BUILD_JOBS", "2")
    cmd = ["cargo", "test", "-p", crate]
    if mod_names:
        cmd.extend(["--lib", "--quiet", "--", *[f"ported::{m}" for m in mod_names]])
    else:
        cmd.append("--quiet")
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = (proc.stdout or "") + "\n" + (proc.stderr or "")
    return proc.returncode == 0, out.strip()


def run_workspace_test() -> tuple[bool, str]:
    env = os.environ.copy()
    env.setdefault("CARGO_BUILD_JOBS", "2")
    proc = subprocess.run(
        ["cargo", "test", "--workspace", "--quiet"],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        timeout=600,
    )
    out = (proc.stdout or "") + "\n" + (proc.stderr or "")
    return proc.returncode == 0, out.strip()


def summarize_cargo_error(output: str) -> str:
    """Return the primary compiler or test failure line from Cargo output."""
    lines = [line.strip() for line in (output or "").splitlines() if line.strip()]
    if not lines:
        return "unknown"
    for line in lines:
        if re.match(r"^error(?:\[E\d+\])?:", line) or re.match(r"^test\s+.+\s+FAILED$", line):
            return line
    for line in lines:
        if "error[E" in line or "error:" in line or "FAILED" in line:
            return line
    return lines[-1]


def _validate_module_code(cand: dict, code: str) -> tuple[str | None, str | None]:
    if not code or not code.strip():
        return None, "empty/unparsed module"
    no_comments = re.sub(r"//.*", "", re.sub(r"/\*.*?\*/", "", code, flags=re.DOTALL))
    if re.search(r"\bunsafe\s*(?:\{|fn\b|impl\b|trait\b|extern\b)", no_comments):
        return None, "contained unsafe syntax"
    if "// JAVA:" not in code:
        code = f"// JAVA: {cand['fqcn']}#class\n" + code
    if "#[cfg(test)]" not in code:
        return None, "missing `#[cfg(test)]` unit tests"
    return code, None


def port_batch(batch: list[dict]) -> tuple[list[dict], list[dict]]:
    """
    Port a 1-to-N same-crate package batch of Java classes.
    Returns `(passed_candidates, failed_candidates)`.
    """
    crate = batch[0]["crate"]
    crate_ctx = read_crate_context(crate)
    remaining = list(batch)
    passed_all: list[dict] = []
    skipped_existing: list[dict] = []
    mod_feedback: dict[str, str] = {}

    try:
        for attempt in range(1, 4):
            if not remaining:
                break

            efforts = ["medium", "high", "max"]
            budgets = [20480, 36864, 65536]

            lead = remaining[0]
            set_worker_activity(lead["fqcn"], crate, attempt, "generating")

            if len(remaining) == 1:
                cand = remaining[0]
                fqcn = cand["fqcn"]
                mod_name = cand["mod"]
                java_src = cand["path"].read_text(encoding="utf-8", errors="ignore")
                system_prompt = (
                    "You are an expert Rust systems engineer porting Minecraft Java Edition 1.21.11 "
                    "to Rust (Phase A: 1:1 faithful behavioral port).\n"
                    "STRICT RULES:\n"
                    "1. Zero `unsafe` code (`crates` use `#![forbid(unsafe_code)]`; do NOT put `#![...]` crate attributes in a submodule).\n"
                    "2. Match Java integer semantics explicitly: `wrapping_add`, `wrapping_sub`, `wrapping_mul`, masked shifts (`& 31`, `& 63`).\n"
                    "3. Match Java `f32`/`f64` operation order identically.\n"
                    f"4. Annotate every ported function/method with `// JAVA: {fqcn}#<methodName>` directly above `pub fn` / `fn`.\n"
                    f"5. Only use `std` and dependencies already declared in `{crate}/Cargo.toml` (plus sibling items from `crate::`).\n"
                    "6. Make the module self-contained without requiring unported external Minecraft classes (define minimal local traits/types if needed).\n"
                    "7. Keep implementations and tests clean and concise: 1:1 faithful without bloated boilerplate; include 3-6 focused `#[cfg(test)] mod tests` covering edge cases, overflow, negative inputs, and float bits.\n"
                    "8. Output ONLY a single ```rust code block.\n"
                )
                fb = mod_feedback.get(mod_name, "")
                prompt = (
                    f"Port `{fqcn}` to a clean, self-contained Rust module `{crate}/src/ported/{mod_name}.rs`.\n\n"
                    f"{crate_ctx}\n\n"
                    f"### Decompiled Java 1.21.11 Source (`{cand['rel']}`):\n```java\n{java_src}\n```\n\n"
                    f"{fb}"
                )
            else:
                system_prompt = (
                    "You are an expert Rust systems engineer porting Minecraft Java Edition 1.21.11 "
                    "to Rust (Phase A: 1:1 faithful behavioral port).\n"
                    "STRICT RULES:\n"
                    "1. Zero `unsafe` code (`crates` use `#![forbid(unsafe_code)]`; do NOT put `#![...]` crate attributes in a submodule).\n"
                    "2. Match Java integer semantics explicitly: `wrapping_add`, `wrapping_sub`, `wrapping_mul`, masked shifts (`& 31`, `& 63`).\n"
                    "3. Match Java `f32`/`f64` operation order identically.\n"
                    "4. Annotate every ported function/method with `// JAVA: <fqcn>#<methodName>` directly above `pub fn` / `fn`.\n"
                    f"5. Only use `std` and dependencies already declared in `{crate}/Cargo.toml` (plus sibling items from `crate::`).\n"
                    "6. Keep EACH module self-contained so it compiles on its own without unported external Minecraft classes (do NOT use `super::other_batch_mod` unless already in `lib.rs`).\n"
                    "7. Keep implementations and tests clean and concise: 1:1 faithful without bloated boilerplate; include 3-6 focused `#[cfg(test)] mod tests` per module covering edge cases, overflow, negative inputs, and float bits.\n"
                    "8. Output ONE ```rust code block per class, each immediately preceded by a line formatted EXACTLY as `// === FILE: <mod_name>.rs ===`.\n"
                )
                sections = []
                for c in remaining:
                    java_src = c["path"].read_text(encoding="utf-8", errors="ignore")
                    fb = mod_feedback.get(c["mod"], "")
                    fb_block = f"\nPrevious error for `{c['mod']}.rs`:\n{fb}\n" if fb else ""
                    sections.append(
                        f"#### Class `{c['fqcn']}` -> marker `// === FILE: {c['mod']}.rs ===` (`{crate}/src/ported/{c['mod']}.rs`)\n"
                        f"```java\n{java_src}\n```{fb_block}"
                    )
                prompt = (
                    f"Port these {len(remaining)} Java classes to separate self-contained Rust modules in `{crate}`:\n\n"
                    f"{crate_ctx}\n\n" + "\n\n".join(sections)
                )

            try:
                reply = deepai_backend.ask(
                    system_prompt,
                    prompt,
                    max_tokens=min(MAX_OUTPUT_TOKENS, budgets[attempt - 1]),
                    effort=efforts[attempt - 1],
                )
            except deepai_backend.QuotaExhausted:
                raise
            except Exception as exc:
                log(f"  [{lead['fqcn']} attempt {attempt}] LLM error: {exc}")
                continue

            parsed = parse_batch_rust_blocks(reply, [c["mod"] for c in remaining])
            valid_items: list[tuple[dict, str]] = []
            next_feedback: dict[str, str] = {}

            for c in remaining:
                raw_code = parsed.get(c["mod"], "")
                clean_code, invalid_reason = _validate_module_code(c, raw_code)
                if clean_code is not None:
                    valid_items.append((c, clean_code))
                else:
                    log(f"  [{c['fqcn']} attempt {attempt}] validation failed: {invalid_reason}")
                    next_feedback[c["mod"]] = (
                        f"Previous attempt for `{c['mod']}.rs` failed validation ({invalid_reason}). "
                        f"Output a valid safe ```rust block preceded by `// === FILE: {c['mod']}.rs ===` with `#[cfg(test)]` unit tests.\n"
                    )

            if not valid_items:
                mod_feedback = next_feedback
                continue

            with repo_write_lock():
                set_worker_activity(lead["fqcn"], crate, attempt, "testing")
                ported_dir = CRATES_DIR / crate / "src" / "ported"
                testable: list[tuple[dict, str]] = []
                for c, code in valid_items:
                    if (ported_dir / f"{c['mod']}.rs").exists():
                        log(f"  [{c['fqcn']}] target module {crate}/src/ported/{c['mod']}.rs already exists; skipping")
                        skipped_existing.append(c)
                        continue
                    testable.append((c, code))

                remaining_after: list[dict] = [
                    c for c in remaining
                    if c["mod"] in next_feedback
                ]

                if len(testable) == 1:
                    c, code = testable[0]
                    backups = wire_ported_modules(crate, [(c["mod"], code)])
                    try:
                        ok, test_out = run_cargo_test(crate, [c["mod"]])
                    except Exception as exc:
                        ok, test_out = False, str(exc)

                    if ok:
                        log(f"  [PASS] {c['fqcn']} -> {crate}/src/ported/{c['mod']}.rs (attempt {attempt}/3)")
                        passed_all.append(c)
                    else:
                        rollback(backups)
                        err_tail = test_out[-3000:]
                        log(f"  [{c['fqcn']} attempt {attempt}] cargo test failed: {summarize_cargo_error(test_out)}")
                        next_feedback[c["mod"]] = (
                            f"### Attempt {attempt} `cargo test -p {crate}` failure:\n"
                            f"```text\n{err_tail}\n```\n"
                            f"Fix every compiler error and failing test while preserving 1:1 Java semantics.\n"
                        )
                        remaining_after.append(c)

                elif len(testable) > 1:
                    batch_mods = [(c["mod"], code) for c, code in testable]
                    batch_backups = wire_ported_modules(crate, batch_mods)
                    try:
                        ok, test_out = run_cargo_test(crate, [m for m, _ in batch_mods])
                    except Exception as exc:
                        ok, test_out = False, str(exc)

                    if ok:
                        names = ", ".join(c["mod"] for c, _ in testable)
                        log(f"  [PASS BATCH x{len(testable)}] {crate}: [{names}] (attempt {attempt}/3)")
                        passed_all.extend(c for c, _ in testable)
                    else:
                        rollback(batch_backups)
                        log(
                            f"  [BATCH SPLIT x{len(testable)}] {crate} combined check failed "
                            f"({summarize_cargo_error(test_out)}); verifying individually"
                        )
                        for c, code in testable:
                            ind_backups = wire_ported_modules(crate, [(c["mod"], code)])
                            try:
                                ind_ok, ind_out = run_cargo_test(crate, [c["mod"]])
                            except Exception as exc:
                                ind_ok, ind_out = False, str(exc)

                            if ind_ok:
                                log(f"  [PASS] {c['fqcn']} -> {crate}/src/ported/{c['mod']}.rs (batch fallback, attempt {attempt}/3)")
                                passed_all.append(c)
                            else:
                                rollback(ind_backups)
                                err_tail = ind_out[-2500:]
                                log(f"  [{c['fqcn']} attempt {attempt}] cargo test failed: {summarize_cargo_error(ind_out)}")
                                next_feedback[c["mod"]] = (
                                    f"### Attempt {attempt} `cargo test -p {crate}` failure:\n"
                                    f"```text\n{err_tail}\n```\n"
                                    f"Fix every compiler error and failing test while preserving 1:1 Java semantics.\n"
                                )
                                remaining_after.append(c)

                remaining = remaining_after
                mod_feedback = next_feedback

            if not remaining:
                break

        return passed_all, remaining + skipped_existing
    finally:
        for c in batch:
            clear_worker_activity(c["fqcn"])


def desired_workers() -> int:
    try:
        v = int(WORKERS_FILE.read_text(encoding="utf-8").strip())
        return max(1, min(24, v))
    except (OSError, ValueError):
        return max(1, min(24, int(os.environ.get("MC_RUST_WORKERS", "12"))))


def run_forever() -> None:
    threading.Thread(target=watchdog_loop, name="watchdog", daemon=True).start()
    log("Starting mc-rust 24/7 Autonomous Porting Harness (DeepSeek V4.1 Flash)...")
    if COMPLETION_FLAG.exists():
        log("COMPLETED_100_PERCENT.flag exists; idling.")
        while True:
            time.sleep(1800)

    while True:
        try:
            try:
                info = deepai_backend.preflight()
                log(
                    f"DeepAI ready: {info['model']} | keys={info['key_count']} | "
                    f"context={info['context_window']} | adaptive effort (med->high->max)"
                )
            except Exception as exc:
                log(f"DeepAI preflight failed ({exc}); retrying in 120s")
                publish_status(state="waiting_api", class_name=None, crate=None, attempt=0)
                time.sleep(120)
                continue

            skips = load_skips()
            parked = {k for k, v in skips.items() if v[0] >= SKIP_AFTER_FAILURES}
            candidates = [c for c in list_candidate_classes() if c["fqcn"] not in parked]

            if not candidates:
                log("No unparked candidates remaining in decompiled set; sleeping 10m")
                publish_status(state="idle", class_name=None, crate=None, attempt=0)
                time.sleep(600)
                continue

            workers = desired_workers()
            work_units = build_work_units(candidates[:400])
            queue = deque(work_units[:100])
            inflight: dict = {}
            inflight_mods: set[tuple[str, str]] = set()
            last_publish = 0.0
            verified_in_wave = 0
            verified_since_ws = 0
            total_verified = sum(1 for r in update_progress.read_jsonl(LEDGER_PATH) if r.get("verified"))
            publish_status(workers=workers, verified_count=total_verified)
            total_batch_classes = sum(len(u) for u in queue)
            log(f"Dispatching wave of {len(queue)} work units ({total_batch_classes} classes) with {workers} parallel workers")

            with ThreadPoolExecutor(max_workers=workers) as pool:
                while queue or inflight:
                    rotations = 0
                    while queue and len(inflight) < workers:
                        batch = queue.popleft()
                        mod_keys = {(c["crate"], c["mod"]) for c in batch}
                        # Keep one module-write at a time per target module filename
                        if mod_keys & inflight_mods:
                            queue.append(batch)
                            rotations += 1
                            if rotations >= len(queue):
                                break
                            continue
                        inflight_mods.update(mod_keys)
                        fut = pool.submit(port_batch, batch)
                        inflight[fut] = (batch, time.time())

                    if not inflight:
                        break
                    done_futs, _ = wait(inflight, return_when=FIRST_COMPLETED)
                    for fut in done_futs:
                        batch, started = inflight.pop(fut)
                        for c in batch:
                            inflight_mods.discard((c["crate"], c["mod"]))
                        elapsed = time.time() - started
                        try:
                            passed_cands, failed_cands = fut.result()
                        except deepai_backend.QuotaExhausted:
                            raise
                        except Exception as exc:
                            log(f"  [{batch[0]['fqcn']}] worker error: {exc}")
                            passed_cands, failed_cands = [], list(batch)

                        for c in passed_cands:
                            fqcn = c["fqcn"]
                            record_work(fqcn, c["crate"], True, elapsed)
                            skips.pop(fqcn, None)
                            verified_in_wave += 1
                            verified_since_ws += 1
                            total_verified += 1

                        for c in failed_cands:
                            fqcn = c["fqcn"]
                            record_work(fqcn, c["crate"], False, elapsed)
                            fails = (skips.get(fqcn) or [0, 0])[0] + 1
                            skips[fqcn] = [fails, time.time()]

                        if passed_cands:
                            publish_status(verified_count=total_verified, workers=workers)
                        save_skips(skips)

                    if verified_since_ws >= 40:
                        with repo_write_lock():
                            ok_ws, ws_out = run_workspace_test()
                        if ok_ws:
                            log(f"Workspace regression test PASS ({total_verified} total verified)")
                        else:
                            log(f"WARN: workspace test failed: {summarize_cargo_error(ws_out)}")
                        verified_since_ws = 0

                    if verified_in_wave >= 10 or (verified_in_wave > 0 and time.time() - last_publish >= 60):
                        msg = f"Port {verified_in_wave} Java class(es) via DeepSeek V4.1 Flash [Phase A]"
                        with repo_write_lock():
                            update_progress.update_all(commit_msg=msg)
                        last_publish = time.time()
                        verified_in_wave = 0

            if verified_since_ws > 0:
                with repo_write_lock():
                    ok_ws, ws_out = run_workspace_test()
                if ok_ws:
                    log(f"Wave end workspace regression test PASS ({total_verified} total verified)")
                else:
                    log(f"WARN: wave end workspace test failed: {summarize_cargo_error(ws_out)}")

            if verified_in_wave > 0:
                msg = f"Port {verified_in_wave} Java class(es) via DeepSeek V4.1 Flash [Phase A]"
                with repo_write_lock():
                    update_progress.update_all(commit_msg=msg)

            time.sleep(2)

        except deepai_backend.QuotaExhausted as exc:
            log(f"DeepAI quota exhausted across all keys: {exc}; parking 1h")
            publish_status(state="quota_cooldown", class_name=None, crate=None, attempt=0)
            time.sleep(3600)
        except KeyboardInterrupt:
            log("Harness interrupted.")
            break
        except Exception as exc:
            log(f"Supervisor error (retrying in 15s): {exc}")
            time.sleep(15)


if __name__ == "__main__":
    run_forever()
