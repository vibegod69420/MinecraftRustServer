# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:24:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 901 / 66676 (1.351%)
- **Mapped Java Classes**: 289 / 7163 (4.035%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 513 mapped methods
- `mc-nbt`: 139 mapped methods
- `mc-protocol`: 207 mapped methods
- `mc-worldgen`: 247 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType` -> `mc-worldgen` (307.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadType` -> `mc-worldgen` (307.5s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundServerLinksPacket` -> `mc-protocol` (230.8s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundDisconnectPacket` -> `mc-protocol` (230.8s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundClearDialogPacket` -> `mc-protocol` (230.8s)
- [PASS] `net.minecraft.network.protocol.common.ServerCommonPacketListener` -> `mc-protocol` (230.8s)
- [PASS] `net.minecraft.core.dispenser.EquipmentDispenseItemBehavior` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.core.dispenser.ShulkerBoxDispenseBehavior` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.core.dispenser.BlockSource` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.core.dispenser.OptionalDispenseItemBehavior` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundResetChatPacket` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.core.component.predicates.WritableBookPredicate` -> `mc-math` (216.6s)
