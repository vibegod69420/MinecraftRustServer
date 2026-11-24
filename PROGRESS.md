# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 18:44:02 UTC
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
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (2986.5s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseChunk` -> `mc-worldgen` (1949.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces` -> `mc-worldgen` (1816.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (1023.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (943.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkHeightAndBiomeFix` -> `mc-math` (1444.4s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (1343.3s)
- [FAIL] `net.minecraft.util.profiling.jfr.JfrProfiler` -> `mc-math` (1321.9s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (1236.4s)
- [FAIL] `net.minecraft.util.parsing.packrat.commands.Grammar` -> `mc-math` (971.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkProtoTickListFix` -> `mc-math` (957.4s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (821.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.GameRuleRegistryFix` -> `mc-math` (808.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix` -> `mc-math` (738.7s)
- [FAIL] `net.minecraft.util.datafix.schemas.V705` -> `mc-math` (706.4s)
