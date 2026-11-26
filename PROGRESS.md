# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 00:38:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9761 / 66676 (14.639%)
- **Mapped Java Classes**: 2207 / 7163 (30.811%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1963 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.schemas.V704` -> `mc-math` (611.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (541.7s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (1550.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (643.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (465.4s)
- [FAIL] `net.minecraft.util.profiling.jfr.parse.JfrStatsParser` -> `mc-math` (515.6s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseChunk` -> `mc-worldgen` (1129.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.SavedDataFeaturePoolElementFix` -> `mc-math` (379.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.TextComponentHoverAndClickEventFix` -> `mc-math` (447.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces` -> `mc-worldgen` (772.6s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (672.9s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (672.9s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (672.9s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (648.6s)
- [FAIL] `net.minecraft.world.level.chunk.UpgradeData` -> `mc-worldgen` (483.3s)
