# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:34:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1305 / 66676 (1.957%)
- **Mapped Java Classes**: 386 / 7163 (5.389%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 588 mapped methods
- `mc-nbt`: 330 mapped methods
- `mc-protocol`: 273 mapped methods
- `mc-worldgen`: 380 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.network.chat.OutgoingChatMessage` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.chat.FontDescription` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.chat.ComponentContents` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.nbt.SnbtOperations` -> `mc-nbt` (376.1s)
