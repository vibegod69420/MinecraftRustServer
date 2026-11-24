# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 17:50:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9652 / 66676 (14.476%)
- **Mapped Java Classes**: 2175 / 7163 (30.364%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1864 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (1356.6s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (2923.1s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (1398.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (936.9s)
- [FAIL] `net.minecraft.util.profiling.jfr.JfrProfiler` -> `mc-math` (1291.3s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (2601.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (598.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkProtoTickListFix` -> `mc-math` (875.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (572.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.GameRuleRegistryFix` -> `mc-math` (868.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix` -> `mc-math` (621.8s)
- [FAIL] `net.minecraft.util.datafix.schemas.V705` -> `mc-math` (445.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (1034.5s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (760.9s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (1992.3s)
