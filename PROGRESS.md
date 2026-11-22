# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:43:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2693 / 66676 (4.039%)
- **Mapped Java Classes**: 688 / 7163 (9.605%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1035 mapped methods
- `mc-nbt`: 845 mapped methods
- `mc-protocol`: 644 mapped methods
- `mc-worldgen`: 685 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket` -> `mc-protocol` (697.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket` -> `mc-protocol` (697.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDeleteChatPacket` -> `mc-protocol` (697.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket` -> `mc-protocol` (697.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootDataType` -> `mc-nbt` (873.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.ContainerComponentManipulator` -> `mc-nbt` (873.0s)
- [PASS] `net.minecraft.world.level.storage.loot.LootParams` -> `mc-nbt` (873.0s)
- [PASS] `net.minecraft.world.level.storage.loot.LootContextUser` -> `mc-nbt` (873.0s)
- [PASS] `net.minecraft.core.RegistrySetBuilder` -> `mc-math` (218.1s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.SlotLoot` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootPoolEntries` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootItem` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.DynamicLoot` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.core.component.predicates.DataComponentPredicate` -> `mc-math` (660.2s)
- [PASS] `net.minecraft.nbt.NbtUtils` -> `mc-nbt` (759.7s)
