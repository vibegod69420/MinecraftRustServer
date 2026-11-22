# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:48:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1722 / 66676 (2.583%)
- **Mapped Java Classes**: 501 / 7163 (6.994%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 693 mapped methods
- `mc-nbt`: 496 mapped methods
- `mc-protocol`: 357 mapped methods
- `mc-worldgen`: 516 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.DefaultedMappedRegistry` -> `mc-math` (584.7s)
- [PASS] `net.minecraft.core.AxisCycle` -> `mc-math` (584.7s)
- [PASS] `net.minecraft.core.FrontAndTop` -> `mc-math` (584.7s)
- [PASS] `net.minecraft.core.NonNullList` -> `mc-math` (584.7s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks` -> `mc-protocol` (364.6s)
- [PASS] `net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket` -> `mc-protocol` (364.6s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket` -> `mc-protocol` (364.6s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket` -> `mc-protocol` (364.6s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemConditionType` -> `mc-nbt` (594.2s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.AnyOfCondition` -> `mc-nbt` (594.2s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemKilledByPlayerCondition` -> `mc-nbt` (594.2s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.ConditionUserBuilder` -> `mc-nbt` (594.2s)
- [PASS] `net.minecraft.core.LayeredRegistryAccess` -> `mc-math` (167.4s)
- [PASS] `net.minecraft.core.BlockBox` -> `mc-math` (167.4s)
- [PASS] `net.minecraft.core.BlockMath` -> `mc-math` (167.4s)
