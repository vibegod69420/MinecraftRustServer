# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:18:26 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3563 / 66676 (5.344%)
- **Mapped Java Classes**: 893 / 7163 (12.467%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1257 mapped methods
- `mc-nbt`: 1130 mapped methods
- `mc-protocol`: 1068 mapped methods
- `mc-worldgen`: 872 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement` -> `mc-worldgen` (238.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement` -> `mc-worldgen` (238.3s)
