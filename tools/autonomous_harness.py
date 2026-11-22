#!/usr/bin/env python3
"""
24/7 Autonomous Phase-A Porting Harness for Minecraft 1.21.11 -> Rust.

Uses DeepSeek V4.1 Flash (`api.deepai.org/deepai_code`, `reasoning_effort: max`, 320K context)
to port server classes from `reference/decompiled/` into `crates/<crate>/src/ported/<module>.rs`
in dependency order, verifying every module with `cargo test --workspace` and publishing
atomic `docs/worker.json` + `docs/progress.json` updates to GitHub.
"""

from __future__ import annotations

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
_STATUS_LOCK = threading.RLock()
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


def read_crate_context(crate: str) -> str:
    lib_rs = CRATES_DIR / crate / "src" / "lib.rs"
    cargo_toml = CRATES_DIR / crate / "Cargo.toml"
    parts = []
    if cargo_toml.exists():
        parts.append(f"### {crate}/Cargo.toml\n```toml\n{cargo_toml.read_text(encoding='utf-8')}\n```")
    if lib_rs.exists():
        parts.append(f"### {crate}/src/lib.rs\n```rust\n{lib_rs.read_text(encoding='utf-8')[:4000]}\n```")
    return "\n\n".join(parts)


def wire_ported_module(crate: str, mod_name: str, code: str) -> list[tuple[Path, str | None]]:
    """
    Write `crates/<crate>/src/ported/<mod_name>.rs`, ensuring `ported/mod.rs` and `lib.rs` wire it.
    Returns list of (path, previous_content_or_None) for clean atomic rollback on failure.
    """
    src_dir = CRATES_DIR / crate / "src"
    ported_dir = src_dir / "ported"
    ported_dir.mkdir(parents=True, exist_ok=True)
    mod_rs = ported_dir / "mod.rs"
    lib_rs = src_dir / "lib.rs"
    target_rs = ported_dir / f"{mod_name}.rs"

    backups: list[tuple[Path, str | None]] = []

    def save_and_write(p: Path, content: str) -> None:
        prev = p.read_text(encoding="utf-8") if p.exists() else None
        backups.append((p, prev))
        p.write_text(content, encoding="utf-8")

    # Disambiguate if module file already exists
    if target_rs.exists():
        raise FileExistsError(f"{target_rs.relative_to(ROOT)} already exists")

    save_and_write(target_rs, code + "\n")

    mod_prev = mod_rs.read_text(encoding="utf-8") if mod_rs.exists() else ""
    decl_line = f"pub mod {mod_name};"
    if decl_line not in mod_prev:
        new_mod = (mod_prev.rstrip() + "\n" + decl_line + "\n").lstrip("\n")
        save_and_write(mod_rs, new_mod)

    lib_prev = lib_rs.read_text(encoding="utf-8")
    if "pub mod ported;" not in lib_prev:
        save_and_write(lib_rs, lib_prev.rstrip() + "\n\npub mod ported;\n")

    return backups


def rollback(backups: list[tuple[Path, str | None]]) -> None:
    for p, prev in reversed(backups):
        if prev is None:
            p.unlink(missing_ok=True)
        else:
            p.write_text(prev, encoding="utf-8")


def run_cargo_test(crate: str) -> tuple[bool, str]:
    env = os.environ.copy()
    env.setdefault("CARGO_BUILD_JOBS", "2")
    proc = subprocess.run(
        ["cargo", "test", "-p", crate, "--quiet"],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        timeout=300,
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


def port_class(candidate: dict) -> bool:
    fqcn = candidate["fqcn"]
    crate = candidate["crate"]
    mod_name = candidate["mod"]
    java_src = candidate["path"].read_text(encoding="utf-8", errors="ignore")
    crate_ctx = read_crate_context(crate)

    system_prompt = (
        "You are an expert Rust systems engineer porting Minecraft Java Edition 1.21.11 "
        "to Rust (Phase A: 1:1 faithful behavioral port).\n"
        "STRICT RULES:\n"
        "1. Zero `unsafe` code (`crates` use `#![forbid(unsafe_code)]`; do NOT put `#![...]` crate attributes in a submodule).\n"
        "2. Match Java integer semantics explicitly: `wrapping_add`, `wrapping_sub`, `wrapping_mul`, masked shifts (`& 31`, `& 63`).\n"
        "3. Match Java `f32`/`f64` operation order identically.\n"
        "4. Annotate every ported function/method with `// JAVA: " + fqcn + "#<methodName>` directly above `pub fn` / `fn`.\n"
        "5. Only use `std` and dependencies already declared in `" + crate + "/Cargo.toml` (plus sibling items from `crate::`).\n"
        "6. Make the module self-contained so it compiles cleanly on its own without requiring unported external Minecraft classes (define clean local traits/types or minimal interfaces if needed).\n"
        "7. Include thorough `#[cfg(test)] mod tests` testing edge cases, overflow, negative inputs, and float bits.\n"
        "8. Output ONLY a single ```rust code block.\n"
    )

    feedback = ""
    budgets = [16384, 32768, 65536]
    efforts = ["high", "max", "max"]

    try:
        for attempt in range(1, 4):
            set_worker_activity(fqcn, crate, attempt, "generating")
            prompt = (
                f"Port `{fqcn}` to a clean, self-contained Rust module `{crate}/src/ported/{mod_name}.rs`.\n\n"
                f"{crate_ctx}\n\n"
                f"### Decompiled Java 1.21.11 Source (`{candidate['rel']}`):\n```java\n{java_src}\n```\n\n"
                f"{feedback}"
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
                log(f"  [{fqcn} attempt {attempt}] LLM error: {exc}")
                continue

            code = extract_rust_block(reply)
            if not code or "unsafe" in re.findall(r"\bunsafe\b", code):
                feedback = "Previous reply was empty or contained `unsafe`; output a single valid safe ```rust block.\n"
                continue
            if "// JAVA:" not in code:
                code = f"// JAVA: {fqcn}#class\n" + code
            if "#[cfg(test)]" not in code:
                feedback = "Previous reply lacked `#[cfg(test)]` tests; include unit tests.\n"
                continue

            with WRITE_LOCK:
                set_worker_activity(fqcn, crate, attempt, "testing")
                try:
                    backups = wire_ported_module(crate, mod_name, code)
                except FileExistsError:
                    log(f"  [{fqcn}] target module {crate}/src/ported/{mod_name}.rs already exists; skipping")
                    return False
                try:
                    ok, test_out = run_cargo_test(crate)
                except Exception as exc:
                    ok, test_out = False, str(exc)

                if ok:
                    log(f"  [PASS] {fqcn} -> {crate}/src/ported/{mod_name}.rs (attempt {attempt}/3)")
                    return True

                rollback(backups)
                err_tail = test_out[-3000:]
                log(f"  [{fqcn} attempt {attempt}] cargo test failed: {summarize_cargo_error(test_out)}")
                feedback = (
                    f"### Attempt {attempt} `cargo test -p {crate}` failure:\n"
                    f"```text\n{err_tail}\n```\n"
                    f"Fix every compiler error and failing test while preserving 1:1 Java semantics.\n"
                )

        return False
    finally:
        clear_worker_activity(fqcn)


def desired_workers() -> int:
    try:
        v = int(WORKERS_FILE.read_text(encoding="utf-8").strip())
        return max(1, min(9, v))
    except (OSError, ValueError):
        return max(1, min(9, int(os.environ.get("MC_RUST_WORKERS", "6"))))


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
                    f"context={info['context_window']} | effort={info['reasoning_effort']}"
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
            queue = deque(candidates[:200])
            inflight: dict = {}
            inflight_mods: set[tuple[str, str]] = set()
            last_publish = 0.0
            verified_in_wave = 0
            total_verified = sum(1 for r in update_progress.read_jsonl(LEDGER_PATH) if r.get("verified"))
            publish_status(workers=workers, verified_count=total_verified)
            log(f"Dispatching wave of {len(queue)} candidates with {workers} parallel workers")

            with ThreadPoolExecutor(max_workers=workers) as pool:
                while queue or inflight:
                    rotations = 0
                    while queue and len(inflight) < workers:
                        cand = queue.popleft()
                        mod_key = (cand["crate"], cand["mod"])
                        # Keep one module-write at a time per target module filename
                        if mod_key in inflight_mods:
                            queue.append(cand)
                            rotations += 1
                            if rotations >= len(queue):
                                break
                            continue
                        inflight_mods.add(mod_key)
                        fut = pool.submit(port_class, cand)
                        inflight[fut] = (cand, time.time())

                    if not inflight:
                        break
                    done_futs, _ = wait(inflight, return_when=FIRST_COMPLETED)
                    for fut in done_futs:
                        cand, started = inflight.pop(fut)
                        inflight_mods.discard((cand["crate"], cand["mod"]))
                        fqcn = cand["fqcn"]
                        try:
                            ok = fut.result()
                        except deepai_backend.QuotaExhausted:
                            raise
                        except Exception as exc:
                            log(f"  [{fqcn}] worker error: {exc}")
                            ok = False

                        record_work(fqcn, cand["crate"], ok, time.time() - started)
                        if ok:
                            skips.pop(fqcn, None)
                            verified_in_wave += 1
                            total_verified += 1
                            publish_status(verified_count=total_verified, workers=workers)
                        else:
                            fails = (skips.get(fqcn) or [0, 0])[0] + 1
                            skips[fqcn] = [fails, time.time()]
                        save_skips(skips)

                    if verified_in_wave > 0 and time.time() - last_publish >= 120:
                        msg = f"Port {verified_in_wave} Java class(es) via DeepSeek V4.1 Flash [Phase A]"
                        with WRITE_LOCK:
                            update_progress.update_all(commit_msg=msg)
                        last_publish = time.time()
                        verified_in_wave = 0

            if verified_in_wave > 0:
                msg = f"Port {verified_in_wave} Java class(es) via DeepSeek V4.1 Flash [Phase A]"
                with WRITE_LOCK:
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
