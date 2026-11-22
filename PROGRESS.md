# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:34:18 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1315 / 66676 (1.972%)
- **Mapped Java Classes**: 390 / 7163 (5.445%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 588 mapped methods
- `mc-nbt`: 341 mapped methods
- `mc-protocol`: 273 mapped methods
- `mc-worldgen`: 380 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.DataVersion` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.world.level.storage.LevelStorageException` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.world.level.storage.LevelDataAndDimensions` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.world.level.storage.WritableLevelData` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.nbt.LongArrayTag` -> `mc-nbt` (529.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.TwoLayersFeatureSize` -> `mc-worldgen` (407.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize` -> `mc-worldgen` (407.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSizeType` -> `mc-worldgen` (407.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.ReplaceBlockConfiguration` -> `mc-worldgen` (194.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.NetherForestVegetationConfig` -> `mc-worldgen` (194.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.RandomPatchConfiguration` -> `mc-worldgen` (194.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SimpleRandomFeatureConfiguration` -> `mc-worldgen` (194.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.EndCityStructure` -> `mc-worldgen` (786.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.IglooStructure` -> `mc-worldgen` (786.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.JungleTempleStructure` -> `mc-worldgen` (786.9s)
