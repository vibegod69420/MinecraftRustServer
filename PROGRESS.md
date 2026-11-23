# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:17:21 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4847 / 66676 (7.269%)
- **Mapped Java Classes**: 1143 / 7163 (15.957%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1731 mapped methods
- `mc-nbt`: 1417 mapped methods
- `mc-protocol`: 1624 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.world.level.storage.loot.LootPool` -> `mc-nbt` (379.7s)
- [PASS] `net.minecraft.util.SortedArraySet` -> `mc-math` (282.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider` -> `mc-worldgen` (677.2s)
