# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:05:26 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4489 / 66676 (6.733%)
- **Mapped Java Classes**: 1091 / 7163 (15.231%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1577 mapped methods
- `mc-nbt`: 1289 mapped methods
- `mc-protocol`: 1490 mapped methods
- `mc-worldgen`: 1233 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket` -> `mc-protocol` (773.1s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket` -> `mc-protocol` (773.1s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundChatPacket` -> `mc-protocol` (773.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundDebugSubscriptionRequestPacket` -> `mc-protocol` (773.1s)
- [PASS] `net.minecraft.util.StringRepresentable` -> `mc-math` (201.7s)
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
