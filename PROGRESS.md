# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 17:04:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9791 / 66676 (14.684%)
- **Mapped Java Classes**: 2209 / 7163 (30.839%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1993 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkHeightAndBiomeFix` -> `mc-math` (4933.1s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (5399.6s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (6252.2s)
- [FAIL] `net.minecraft.util.parsing.packrat.CachedParseState` -> `mc-math` (4653.3s)
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (3062.0s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (4964.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (2763.3s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (4480.4s)
- [FAIL] `net.minecraft.util.profiling.jfr.JfrProfiler` -> `mc-math` (3079.0s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (3924.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (3264.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (2508.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (3281.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (1447.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (850.3s)
