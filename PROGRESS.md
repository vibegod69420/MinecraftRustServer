# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 09:44:03 UTC
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
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (587.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (1665.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.GameRuleRegistryFix` -> `mc-math` (385.4s)
- [FAIL] `net.minecraft.util.profiling.metrics.profiling.ActiveMetricsRecorder` -> `mc-math` (1580.4s)
- [FAIL] `net.minecraft.util.datafix.schemas.V704` -> `mc-math` (464.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkTicketUnpackPosFix` -> `mc-math` (1263.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.AbstractBlockPropertyFix` -> `mc-math` (1263.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityFallDistanceFloatToDoubleFix` -> `mc-math` (1263.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.BlockPropertyRenameAndFix` -> `mc-math` (1263.2s)
- [FAIL] `net.minecraft.util.parsing.packrat.commands.Grammar` -> `mc-math` (1204.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.SavedDataFeaturePoolElementFix` -> `mc-math` (497.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (345.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (848.0s)
- [FAIL] `net.minecraft.world.level.chunk.UpgradeData` -> `mc-worldgen` (736.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.TextComponentHoverAndClickEventFix` -> `mc-math` (599.4s)
