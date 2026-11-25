# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 10:16:02 UTC
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
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (3555.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (2112.9s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (1287.7s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (2900.4s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (2855.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix` -> `mc-math` (1118.1s)
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (517.7s)
- [FAIL] `net.minecraft.util.datafix.schemas.V705` -> `mc-math` (939.1s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (2628.3s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (2619.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (474.2s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (1059.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.StatsCounterFix` -> `mc-math` (614.3s)
- [FAIL] `net.minecraft.util.profiling.jfr.parse.JfrStatsParser` -> `mc-math` (1161.7s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (1808.7s)
