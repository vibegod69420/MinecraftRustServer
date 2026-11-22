# Architecture & Engineering Decisions (`DECISIONS.md`)

Log of all non-trivial engineering decisions, alternatives considered, and empirical findings.

---

## D001: AWS Development, Oracle & Benchmark Instance
- **Date**: 2026-11-22
- **Decision**: Provision `mc-rust-dev` in `us-east-1a` using `t4g.small` (`arm64` Graviton2, 2 vCPU, 2 GiB RAM, 40 GiB `gp3`, 4 GiB `/swapfile`, Ubuntu 24.04 `ami-04512f58a86d569d1`) via CloudFormation (`infra/template.yaml`) in VPC `vpc-0ab53fc97a8b5dcfe` / subnet `subnet-09039b1102fb25f32`, reusing keypair `mc-t4g-key`.
- **Rationale**:
  - Initial `t4g.large` deployment was rejected by AWS with `"The specified instance type is not eligible for Free Tier"`. `describe-instance-types --filters free-tier-eligible=true` showed `t4g.small`/`t9g.small` (2 GiB `arm64`) and `m7i-flex.large` (8 GiB `x86_64`) as eligible.
  - Choosing `t4g.small` keeps the exact `arm64` CPU architecture of the existing `mc-t4g-paper-1.21.11` (`t4g.small`) instance so HotSpot `Math.*` floating-point intrinsics match target hardware, while adding a 4 GiB swap file + `CARGO_BUILD_JOBS=2` avoids OOM during Rust/Cargo compilation.
  - Restricts SSH (22) and MC test port (25565) to `190.221.78.169/32`.

## D002: Reference Source Selection (Vanilla First, Paper Flagged)
- **Date**: 2026-11-22
- **Decision**: Use the official Mojang `1.21.11` server jar (extracted from bundler `META-INF/versions`) + official Mojang `server_mappings` + Vineflower decompilation as the primary ground-truth reference for all subsystems.
- **Rationale**:
  - Default target behavior is **vanilla 1.21.11**. Paper introduces behavioral changes (chunk loading, redstone timings, entity activation range, async optimizations) that depend on configuration.
  - Paper-specific behavior will be added behind an explicit config flag in later milestones and verified against the Paper oracle separately.
  - All data tables (registries, packets, block states, commands, tags, worldgen JSON) are generated via `net.minecraft.data.Main` directly from the official jar rather than hand-transcribed.

## D003: Workspace Crate Decomposition (Phase A)
- **Date**: 2026-11-22
- **Decision**: Start with 6 focused crates matching dependency order:
  1. `mc-math`: Random sources (`LegacyRandomSource`, `Xoroshiro128PlusPlus`, positional factories), `Mth`, noise (`ImprovedNoise`, `PerlinNoise`, `NormalNoise`), `BlockPos`/`SectionPos`, `SimpleBitStorage`, RNG trace hook.
  2. `mc-nbt`: Binary NBT reader/writer, canonical structural comparator/hasher, SNBT/region scaffolding.
  3. `mc-data`: Build/codegen-driven registries, packet IDs, block states, properties generated from vanilla `reports/`.
  4. `mc-protocol`: VarInt/VarLong, packet framing, Handshake/Login/Configuration/Play codecs (Milestone 2).
  5. `mc-worldgen`: Noise routers, density functions, biomes, chunk generation stages (Milestone 2+).
  6. `mc-harness`: Differential test runner, Merkle state comparator, shrinker CLI.

## D004: Safe-by-Default Rust & Integer/Float Semantics
- **Date**: 2026-11-22
- **Decision**:
  - Zero `unsafe` in Phase A crates (`#![forbid(unsafe_code)]` where feasible).
  - All integer arithmetic in ported Java math/RNG/hashing uses explicit `wrapping_add`, `wrapping_sub`, `wrapping_mul`, `wrapping_neg`, and masked shifts (`>> (n & 31)` / `>> (n & 63)`) to match JVM bytecode semantics and avoid debug-mode overflow panics.
  - Floating-point comparisons in tests use `.to_bits()` (exact IEEE-754 representation, including `-0.0` vs `+0.0` where applicable).

## D005: Deterministic JVM Identity Hash Codes
- **Date**: 2026-11-22
- **Decision**: Launch the Java oracle with `-XX:+UnlockExperimentalVMOptions -XX:hashCode=2`.
- **Rationale**: Verified on Temurin OpenJDK 21.0.12 that `-XX:hashCode=2` forces `System.identityHashCode` to return constant `1`, eliminating ASLR/thread-order nondeterminism in `IdentityHashMap` / `HashSet` iterations when objects rely on identity hash codes.

## D006: Pinned Oracle Architecture & `Math.*` vs `StrictMath.*` Audit
- **Date**: 2026-11-22
- **Decision**: Pin the authoritative oracle architecture to `aarch64` (Ubuntu 24.04 OpenJDK `21.0.13`, matching `mc-rust-dev` and `mc-t4g-paper-1.21.11`) and implement deterministic pure-Rust `fdlibm` (`crates/mc-math/src/fdlibm.rs`) matching `aarch64` OpenJDK behavior.
- **Empirical Audit (1,000,000 seeded samples per op)**:
  - **Windows `amd64` (Temurin `21.0.12+8`)**: `sin=33,867`, `cos=33,165`, `pow=97,270`, `exp=95,563`, `log=19,691`, `sqrt=0`, `atan2=0` diffs between `Math.*` and `StrictMath.*`.
  - **Linux `aarch64` (Ubuntu `21.0.13.1+2`)**: `sin=72,060`, `cos=71,883`, `pow=0`, `exp=0`, `log=0`, `sqrt=0`, `atan2=0` diffs between `Math.*` and `StrictMath.*`.
  - **Table checks on `aarch64`**: All 65,536 `Mth.SIN` `f32` LUT entries match `StrictMath.sin` (`0` diffs); 256/257 `Mth.COS_TAB` entries match `StrictMath`, with only index `205` (`4603570143850921360` vs `4603570143850921359`) explicitly pinned in `crates/mc-math/src/mth.rs`.

## D007: Zero-Redeploy Live Progress Dashboard
- **Date**: 2026-11-22
- **Decision**: Serve `docs/index.html` once via GitHub Pages (workflow triggers only on `paths: ['docs/index.html', '.github/workflows/deploy-pages.yml']`) and fetch `docs/progress.json` + `docs/worker.json` client-side from `raw.githubusercontent.com/blubbersy-hub/MinecraftRustServer/main/docs/` every 15 seconds.
- **Rationale**: Progress updates on `main` immediately go live without wasting GitHub Actions Pages build minutes or queuing redeploys.
