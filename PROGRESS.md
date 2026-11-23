# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:54:04 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4283 / 66676 (6.424%)
- **Mapped Java Classes**: 1045 / 7163 (14.589%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1533 mapped methods
- `mc-nbt`: 1227 mapped methods
- `mc-protocol`: 1399 mapped methods
- `mc-worldgen`: 1150 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.entries.NestedLootTable` -> `mc-nbt` (243.0s)
- [PASS] `net.minecraft.world.level.storage.CommandStorage` -> `mc-nbt` (174.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.CoralClawFeature` -> `mc-worldgen` (984.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockColumnFeature` -> `mc-worldgen` (984.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.KelpFeature` -> `mc-worldgen` (984.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.SeagrassFeature` -> `mc-worldgen` (984.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (913.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.CreakingHeartDecorator` -> `mc-worldgen` (913.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.LeaveVineDecorator` -> `mc-worldgen` (913.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.HugeBrownMushroomFeature` -> `mc-worldgen` (407.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FossilFeatureConfiguration` -> `mc-worldgen` (407.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlueIceFeature` -> `mc-worldgen` (407.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket` -> `mc-protocol` (575.3s)
