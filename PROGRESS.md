# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:03:41 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4469 / 66676 (6.703%)
- **Mapped Java Classes**: 1085 / 7163 (15.147%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1563 mapped methods
- `mc-nbt`: 1289 mapped methods
- `mc-protocol`: 1490 mapped methods
- `mc-worldgen`: 1214 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.world.level.storage.DerivedLevelData` -> `mc-nbt` (501.4s)
- [PASS] `net.minecraft.world.level.storage.loot.LootContextArg` -> `mc-nbt` (762.6s)
- [PASS] `net.minecraft.util.KeyframeTrack` -> `mc-math` (733.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer` -> `mc-worldgen` (595.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer` -> `mc-worldgen` (595.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.CopyBlockState` -> `mc-nbt` (543.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature` -> `mc-worldgen` (707.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (707.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BonusChestFeature` -> `mc-worldgen` (707.1s)
- [PASS] `net.minecraft.util.debug.DebugSubscriptions` -> `mc-math` (475.8s)
- [PASS] `net.minecraft.util.KeyframeTrackSampler` -> `mc-math` (659.7s)
