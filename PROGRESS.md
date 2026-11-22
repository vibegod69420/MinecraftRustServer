# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:44:29 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2733 / 66676 (4.099%)
- **Mapped Java Classes**: 698 / 7163 (9.745%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1043 mapped methods
- `mc-nbt`: 861 mapped methods
- `mc-protocol`: 644 mapped methods
- `mc-worldgen`: 701 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyNameFunction` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ToggleTooltips` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetCustomDataFunction` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.core.registries.Registries` -> `mc-math` (165.1s)
- [FAIL] `net.minecraft.core.Registry` -> `mc-math` (955.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket` -> `mc-protocol` (697.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPlayerInventoryPacket` -> `mc-protocol` (697.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDeleteChatPacket` -> `mc-protocol` (697.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket` -> `mc-protocol` (697.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootDataType` -> `mc-nbt` (873.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.ContainerComponentManipulator` -> `mc-nbt` (873.0s)
- [PASS] `net.minecraft.world.level.storage.loot.LootParams` -> `mc-nbt` (873.0s)
- [PASS] `net.minecraft.world.level.storage.loot.LootContextUser` -> `mc-nbt` (873.0s)
- [PASS] `net.minecraft.core.RegistrySetBuilder` -> `mc-math` (218.1s)
