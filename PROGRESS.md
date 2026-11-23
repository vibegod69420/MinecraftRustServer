# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:18:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3531 / 66676 (5.296%)
- **Mapped Java Classes**: 887 / 7163 (12.383%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1257 mapped methods
- `mc-nbt`: 1130 mapped methods
- `mc-protocol`: 1018 mapped methods
- `mc-worldgen`: 872 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.network.protocol.game.ServerboundSetBeaconPacket` -> `mc-protocol` (533.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket` -> `mc-protocol` (533.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundContainerClosePacket` -> `mc-protocol` (533.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket` -> `mc-protocol` (533.0s)
