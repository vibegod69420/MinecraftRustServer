# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 07:58:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9675 / 66676 (14.511%)
- **Mapped Java Classes**: 2180 / 7163 (30.434%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1891 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (3473.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (433.3s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (3624.1s)
- [FAIL] `net.minecraft.util.datafix.schemas.V705` -> `mc-math` (583.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (475.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (4242.2s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (1079.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.GameRuleRegistryFix` -> `mc-math` (940.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.TextComponentHoverAndClickEventFix` -> `mc-math` (1654.5s)
- [FAIL] `net.minecraft.util.profiling.jfr.parse.JfrStatsParser` -> `mc-math` (1371.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.StatsCounterFix` -> `mc-math` (544.3s)
- [FAIL] `net.minecraft.util.profiling.metrics.profiling.ActiveMetricsRecorder` -> `mc-math` (1622.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (554.0s)
- [FAIL] `net.minecraft.util.datafix.schemas.V704` -> `mc-math` (466.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.AbstractBlockPropertyFix` -> `mc-math` (1475.7s)
