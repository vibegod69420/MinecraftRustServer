# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 06:55:21 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7873 / 66676 (11.808%)
- **Mapped Java Classes**: 1655 / 7163 (23.105%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2115 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2592 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.util.parsing.packrat.DelayedException` -> `mc-math` (184.2s)
- [PASS] `net.minecraft.util.parsing.packrat.ParseState` -> `mc-math` (184.2s)
- [PASS] `net.minecraft.util.parsing.packrat.Control` -> `mc-math` (184.2s)
- [PASS] `net.minecraft.util.parsing.packrat.NamedRule` -> `mc-math` (184.2s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (836.0s)
