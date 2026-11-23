# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:29:47 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5036 / 66676 (7.553%)
- **Mapped Java Classes**: 1176 / 7163 (16.418%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1777 mapped methods
- `mc-nbt`: 1492 mapped methods
- `mc-protocol`: 1725 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket` -> `mc-protocol` (728.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetScorePacket` -> `mc-protocol` (728.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket` -> `mc-protocol` (728.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRotateHeadPacket` -> `mc-protocol` (728.7s)
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
