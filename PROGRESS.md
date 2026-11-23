# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:16:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7881 / 66676 (11.820%)
- **Mapped Java Classes**: 1656 / 7163 (23.119%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2115 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2601 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (1720.3s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (1578.8s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (1004.0s)
- [PASS] `net.minecraft.world.level.chunk.storage.EntityStorage` -> `mc-worldgen` (1375.7s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (1136.6s)
- [PASS] `net.minecraft.world.level.chunk.ChunkSource` -> `mc-worldgen` (526.3s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (433.5s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionFileVersion` -> `mc-worldgen` (269.0s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileStorage` -> `mc-worldgen` (219.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RecreatingSimpleRegionStorage` -> `mc-worldgen` (219.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.ChunkIOErrorReporter` -> `mc-worldgen` (219.5s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.ChunkIdentification` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.PacketIdentification` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TickTimeStat` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.CpuLoadStat` -> `mc-math` (125.4s)
