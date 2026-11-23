# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:18:39 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4874 / 66676 (7.310%)
- **Mapped Java Classes**: 1150 / 7163 (16.055%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1740 mapped methods
- `mc-nbt`: 1437 mapped methods
- `mc-protocol`: 1624 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.NativeModuleLister` -> `mc-math` (386.3s)
- [FAIL] `net.minecraft.util.PlaceholderLookupProvider` -> `mc-math` (924.8s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction` -> `mc-nbt` (653.0s)
- [PASS] `net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement` -> `mc-worldgen` (1337.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement` -> `mc-worldgen` (1337.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.PlacementModifierType` -> `mc-worldgen` (1337.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction` -> `mc-nbt` (301.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetTitlesAnimationPacket` -> `mc-protocol` (287.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundAnimatePacket` -> `mc-protocol` (287.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket` -> `mc-protocol` (287.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket` -> `mc-protocol` (287.7s)
- [PASS] `net.minecraft.util.NullOps` -> `mc-math` (470.0s)
- [PASS] `net.minecraft.world.level.levelgen.NoiseSettings` -> `mc-worldgen` (246.9s)
- [PASS] `net.minecraft.world.level.levelgen.NoiseRouter` -> `mc-worldgen` (246.9s)
- [PASS] `net.minecraft.world.level.levelgen.GeodeBlockSettings` -> `mc-worldgen` (246.9s)
