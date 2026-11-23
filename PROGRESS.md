# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:23:54 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4976 / 66676 (7.463%)
- **Mapped Java Classes**: 1167 / 7163 (16.292%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1749 mapped methods
- `mc-nbt`: 1492 mapped methods
- `mc-protocol`: 1681 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.StringDecomposer` -> `mc-math` (1212.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.parameters.LootContextParamSets` -> `mc-nbt` (604.0s)
- [PASS] `net.minecraft.world.level.storage.LevelSummary` -> `mc-nbt` (223.9s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ApplyBonusCount` -> `mc-nbt` (467.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTagQueryPacket` -> `mc-protocol` (847.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetHealthPacket` -> `mc-protocol` (847.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatCommandSignedPacket` -> `mc-protocol` (847.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket` -> `mc-protocol` (847.5s)
- [PASS] `net.minecraft.util.NativeModuleLister` -> `mc-math` (386.3s)
- [FAIL] `net.minecraft.util.PlaceholderLookupProvider` -> `mc-math` (924.8s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyComponentsFunction` -> `mc-nbt` (653.0s)
- [PASS] `net.minecraft.world.level.levelgen.placement.EnvironmentScanPlacement` -> `mc-worldgen` (1337.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement` -> `mc-worldgen` (1337.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.PlacementModifierType` -> `mc-worldgen` (1337.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyCustomDataFunction` -> `mc-nbt` (301.0s)
