# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:52:13 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4248 / 66676 (6.371%)
- **Mapped Java Classes**: 1038 / 7163 (14.491%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1518 mapped methods
- `mc-nbt`: 1200 mapped methods
- `mc-protocol`: 1399 mapped methods
- `mc-worldgen`: 1150 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (913.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.CreakingHeartDecorator` -> `mc-worldgen` (913.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.LeaveVineDecorator` -> `mc-worldgen` (913.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.HugeBrownMushroomFeature` -> `mc-worldgen` (407.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FossilFeatureConfiguration` -> `mc-worldgen` (407.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlueIceFeature` -> `mc-worldgen` (407.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.world.level.levelgen.presets.WorldPreset` -> `mc-worldgen` (244.7s)
- [FAIL] `net.minecraft.core.MappedRegistry` -> `mc-math` (1185.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.SpringFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ReplaceBlobsFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockPileFeature` -> `mc-worldgen` (1126.5s)
