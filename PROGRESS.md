# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:40:27 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2657 / 66676 (3.985%)
- **Mapped Java Classes**: 676 / 7163 (9.437%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 993 mapped methods
- `mc-nbt`: 845 mapped methods
- `mc-protocol`: 644 mapped methods
- `mc-worldgen`: 685 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.NbtUtils` -> `mc-nbt` (759.7s)
- [PASS] `net.minecraft.core.cauldron.CauldronInteraction` -> `mc-math` (215.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureSet` -> `mc-worldgen` (257.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData` -> `mc-worldgen` (257.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.BuiltinStructureSets` -> `mc-worldgen` (257.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.SinglePieceStructure` -> `mc-worldgen` (257.3s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (688.9s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider` -> `mc-nbt` (395.8s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProviders` -> `mc-nbt` (395.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.LavaSubmergedBlockProcessor` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.LinearPosTest` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.ProtectedBlockProcessor` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.placement.HeightRangePlacement` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement` -> `mc-worldgen` (598.4s)
