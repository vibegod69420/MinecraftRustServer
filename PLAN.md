# Minecraft 1.21.11 Server Port to Rust — Master Plan

## Target & Strategy
- **Target Version**: Minecraft Java Edition `1.21.11` (official Mojang server jar, vanilla behavior by default; Paper-specific behavior behind an explicit config flag).
- **Hardware Goal**: Run smoothly on `1 vCPU / ~1 GB RAM` with zero observable differences from the reference server.
- **Two-Phase Rule**:
  - **Phase A (Faithful Port)**: Translate algorithms 1:1 with exact operation order, iteration order, float operation order, wrapping integer semantics, and masked shifts. Zero redesigns or optimizations until equivalence is verified against the oracle.
  - **Phase B (Optimization)**: Only after a subsystem passes the full differential verification suite with zero diffs, optimize (compact storage, arenas, SIMD, order-independent parallelism). Every optimization must keep the full differential suite green.

## Subsystem Inventory (Ranked by Dependency & Risk)

| Priority | Subsystem | Crate(s) | Key Dependencies | Risk Level | Verification Methods |
|---|---|---|---|---|---|
| 1 | Deterministic Oracle & Harness Skeleton | `oracle/`, `crates/mc-harness`, `scenarios/` | JDK 21, ByteBuddy/ASM agent | Critical | Method 0 (self-test), Method 2 (RNG trace), Method 5 (Merkle) |
| 2 | RNG, Math, Noise, BitPacking | `crates/mc-math` | None | High (float/shift precision) | Method 1 (golden vectors), Math vs StrictMath audit |
| 3 | NBT & Anvil / Region / `level.dat` Formats | `crates/mc-nbt` | `mc-math` | Medium | Method 1, Method 3 (canonical NBT + cross-load) |
| 4 | Registries, Blocks, States, Packets, Data Packs | `crates/mc-data` | `net.minecraft.data.Main` reports | High (ID drift) | Codegen from official vanilla reports |
| 5 | Protocol, Framing, Handshake/Login/Config/Play | `crates/mc-protocol` | `mc-data`, `mc-nbt` | High | Method 6 (scripted headless client & decoded packet diff) |
| 6 | World Generation (per ChunkStatus stage) | `crates/mc-worldgen` | `mc-math`, `mc-data`, `mc-nbt` | Very High (float & RNG order) | Method 4 (stage-by-stage chunk diff across seeds & dimensions) |
| 7 | Chunks, PalettedContainer, Heightmaps, Lighting | `mc-world` (Milestone 2+) | `mc-math`, `mc-nbt`, `mc-data` | High | Method 4, Method 5 (Merkle bisection) |
| 8 | Tick Loop, Block Updates, Redstone, Fluids, Scheduled Ticks | `mc-server` (Milestone 2+) | `mc-world`, `mc-data` | Very High (order sensitivity) | Method 5, Method 7 (redstone/fluid scenarios), Method 8 (fuzz) |
| 9 | Entities, Mob AI, Pathfinding, Spawning, Combat, Explosions | `mc-entity` (Milestone 3+) | `mc-world`, `mc-math` | Very High | Method 5, Method 7, Method 8 |
| 10 | Players, Containers/Inventories, Villagers, Trading | `mc-server` (Milestone 3+) | `mc-protocol`, `mc-entity` | High | Method 6, Method 7 |
| 11 | Commands, Brigadier, Loot Tables, Advancements, Scoreboard | `mc-commands` (Milestone 4+) | `mc-data`, `mc-world`, `mc-entity` | Medium-High | Method 6, Method 7 |
| 12 | Coverage, Mutation Testing, Soak, Real Client, Phase B Benchmarks | All | Full server + harness | Continuous | Methods 9, 10, 11, 12 |

## Verification Pipeline (Methods 0–12)
0. **Step 0 — Deterministic Oracle Self-Test**: Freeze ticks (`/tick freeze` + `/tick step N`), seed/patch unseeded RNG (`RandomSupport.generateUniqueSeed`, `UUID.randomUUID`, unseeded `java.util.Random`), pin `-Dmax.bg.threads=1` and JVM hashCode behavior, run twice from scratch, assert identical state hashes and RNG traces.
1. **Method 1 — Golden Vectors**: Generate bit-exact `(input, output)` vectors from JVM for RNGs, `Mth`, noise, `BlockPos`, `SimpleBitStorage`, `NbtIo`; verify `Math.*` vs `StrictMath.*` on target architecture (`arm64` + `x86_64`).
2. **Method 2 — RNG Call-Trace Diff**: Stream-tagged RNG call logging in both Java agent and Rust `mc-math` to pinpoint first divergence.
3. **Method 3 — Data Formats**: Canonical structural NBT comparison and bidirectional cross-loading (`Java <-> Rust`).
4. **Method 4 — Worldgen Per-Stage**: Compare chunks at every `ChunkStatus` stage across all 3 dimensions and many seeds.
5. **Method 5 — Canonical State Hashing & Merkle Bisection**: `section -> chunk -> dimension -> world` Merkle tree + tick-order logging.
6. **Method 6 — Protocol Diff**: Scripted Rust client comparing decoded outgoing packets per tick.
7. **Method 7 — Scenario Suite**: Vanilla structures/tests, redstone contraptions, fluids, mobs, commands.
8. **Method 8 — Differential Fuzzing & Shrinking**: Random scenario generator + delta-debugging shrinker.
9. **Method 9 — Reference Coverage**: JaCoCo agent coverage mapped against `MAPPING.md` and `COVERAGE.md`.
10. **Method 10 — Mutation Testing**: `cargo-mutants` against verified Rust code.
11. **Method 11 — Soak & Real Client**: Multi-day runs + vanilla `1.21.11` client verification.
12. **Method 12 — Benchmarks**: MSPT, RSS, startup, chunk-gen, per-player cost on `1 vCPU / 1 GB RAM`.
