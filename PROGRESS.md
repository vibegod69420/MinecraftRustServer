# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:20:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3592 / 66676 (5.387%)
- **Mapped Java Classes**: 900 / 7163 (12.565%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1257 mapped methods
- `mc-nbt`: 1147 mapped methods
- `mc-protocol`: 1068 mapped methods
- `mc-worldgen`: 895 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.login.ClientboundLoginDisconnectPacket` -> `mc-protocol` (234.0s)
- [PASS] `net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket` -> `mc-protocol` (234.0s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundStoreCookiePacket` -> `mc-protocol` (608.8s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket` -> `mc-protocol` (608.8s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket` -> `mc-protocol` (608.8s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundShowDialogPacket` -> `mc-protocol` (608.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerCombatEndPacket` -> `mc-protocol` (260.9s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetCreativeModeSlotPacket` -> `mc-protocol` (260.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTickingStatePacket` -> `mc-protocol` (260.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTestInstanceBlockStatus` -> `mc-protocol` (260.9s)
- [FAIL] `net.minecraft.world.level.storage.LevelData` -> `mc-nbt` (361.6s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (361.6s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (361.6s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemBlockStatePropertyCondition` -> `mc-nbt` (175.4s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemConditions` -> `mc-nbt` (175.4s)
