# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 14:04:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9767 / 66676 (14.648%)
- **Mapped Java Classes**: 2208 / 7163 (30.825%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1969 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (321.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.SavedDataFeaturePoolElementFix` -> `mc-math` (569.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (1646.1s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (1630.8s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (1391.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (1342.6s)
- [FAIL] `net.minecraft.world.level.levelgen.blending.Blender` -> `mc-worldgen` (1105.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.AbstractBlockPropertyFix` -> `mc-math` (609.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkTicketUnpackPosFix` -> `mc-math` (609.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityFallDistanceFloatToDoubleFix` -> `mc-math` (609.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.BlockPropertyRenameAndFix` -> `mc-math` (609.7s)
- [FAIL] `net.minecraft.util.parsing.packrat.commands.Grammar` -> `mc-math` (337.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (4212.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (3381.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces` -> `mc-worldgen` (3245.1s)
