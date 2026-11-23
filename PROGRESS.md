# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 06:40:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7837 / 66676 (11.754%)
- **Mapped Java Classes**: 1647 / 7163 (22.993%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2115 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2555 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.profiling.jfr.stats.ChunkIdentification` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.PacketIdentification` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TickTimeStat` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.CpuLoadStat` -> `mc-math` (125.4s)
- [PASS] `net.minecraft.util.parsing.packrat.DelayedException` -> `mc-math` (184.2s)
- [PASS] `net.minecraft.util.parsing.packrat.ParseState` -> `mc-math` (184.2s)
- [PASS] `net.minecraft.util.parsing.packrat.Control` -> `mc-math` (184.2s)
- [PASS] `net.minecraft.util.parsing.packrat.NamedRule` -> `mc-math` (184.2s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (836.0s)
- [PASS] `net.minecraft.world.level.chunk.storage.SimpleRegionStorage` -> `mc-worldgen` (1089.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.EntityStorage` -> `mc-worldgen` (687.4s)
- [PASS] `net.minecraft.world.level.chunk.ImposterProtoChunk` -> `mc-worldgen` (615.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces` -> `mc-worldgen` (2062.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileVersion` -> `mc-worldgen` (610.1s)
- [PASS] `net.minecraft.world.level.chunk.DataLayer` -> `mc-worldgen` (646.5s)
