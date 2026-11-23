# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:21:14 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3633 / 66676 (5.449%)
- **Mapped Java Classes**: 912 / 7163 (12.732%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1280 mapped methods
- `mc-nbt`: 1147 mapped methods
- `mc-protocol`: 1102 mapped methods
- `mc-worldgen`: 895 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (380.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer` -> `mc-nbt` (380.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.TagEntry` -> `mc-nbt` (380.4s)
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
