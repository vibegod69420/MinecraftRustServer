# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:28:11 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4994 / 66676 (7.490%)
- **Mapped Java Classes**: 1171 / 7163 (16.348%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1767 mapped methods
- `mc-nbt`: 1492 mapped methods
- `mc-protocol`: 1681 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.FileUtil` -> `mc-math` (897.7s)
- [PASS] `net.minecraft.util.SpawnUtil` -> `mc-math` (1555.9s)
- [PASS] `net.minecraft.util.HttpUtil` -> `mc-math` (668.0s)
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
