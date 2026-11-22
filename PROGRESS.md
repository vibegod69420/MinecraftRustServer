# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:32:56 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1270 / 66676 (1.905%)
- **Mapped Java Classes**: 381 / 7163 (5.319%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 588 mapped methods
- `mc-nbt`: 304 mapped methods
- `mc-protocol`: 273 mapped methods
- `mc-worldgen`: 367 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.nbt.visitors.CollectToTag` -> `mc-nbt` (257.9s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset` -> `mc-worldgen` (267.4s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLayerInfo` -> `mc-worldgen` (267.4s)
- [PASS] `net.minecraft.nbt.StringTag` -> `mc-nbt` (274.7s)
