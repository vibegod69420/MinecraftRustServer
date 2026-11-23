# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:23:13 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3668 / 66676 (5.501%)
- **Mapped Java Classes**: 921 / 7163 (12.858%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1299 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1102 mapped methods
- `mc-worldgen`: 910 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetContainerLootTable` -> `mc-nbt` (402.0s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction` -> `mc-nbt` (402.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.HugeFungusConfiguration` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockBlobFeature` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ConfiguredFeature` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.CoralMushroomFeature` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.util.ClassTreeIdRegistry` -> `mc-math` (449.0s)
- [PASS] `net.minecraft.util.TriState` -> `mc-math` (449.0s)
- [PASS] `net.minecraft.util.ListAndDeque` -> `mc-math` (449.0s)
- [PASS] `net.minecraft.util.Brightness` -> `mc-math` (449.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPaddleBoatPacket` -> `mc-protocol` (303.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetCameraPacket` -> `mc-protocol` (303.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMoveVehiclePacket` -> `mc-protocol` (303.7s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSelectBundleItemPacket` -> `mc-protocol` (303.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (380.4s)
