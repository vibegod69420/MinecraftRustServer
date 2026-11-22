# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:45:55 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1698 / 66676 (2.547%)
- **Mapped Java Classes**: 492 / 7163 (6.869%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 679 mapped methods
- `mc-nbt`: 496 mapped methods
- `mc-protocol`: 357 mapped methods
- `mc-worldgen`: 494 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.network.PacketSendListener` -> `mc-protocol` (143.4s)
- [PASS] `net.minecraft.network.RateKickingConnection` -> `mc-protocol` (143.4s)
- [PASS] `net.minecraft.network.ProtocolInfo` -> `mc-protocol` (143.4s)
- [PASS] `net.minecraft.network.HandlerNames` -> `mc-protocol` (143.4s)
