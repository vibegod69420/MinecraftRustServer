# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:22:51 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 856 / 66676 (1.284%)
- **Mapped Java Classes**: 285 / 7163 (3.979%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 513 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 207 mapped methods
- `mc-worldgen`: 245 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.dispenser.EquipmentDispenseItemBehavior` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.core.dispenser.ShulkerBoxDispenseBehavior` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.core.dispenser.BlockSource` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.core.dispenser.OptionalDispenseItemBehavior` -> `mc-math` (227.5s)
- [PASS] `net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundResetChatPacket` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientConfigurationPacketListener` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.network.protocol.configuration.ServerConfigurationPacketListener` -> `mc-protocol` (223.1s)
- [PASS] `net.minecraft.core.component.predicates.WritableBookPredicate` -> `mc-math` (216.6s)
- [PASS] `net.minecraft.core.component.predicates.TrimPredicate` -> `mc-math` (216.6s)
- [PASS] `net.minecraft.core.component.predicates.FireworksPredicate` -> `mc-math` (216.6s)
- [PASS] `net.minecraft.core.component.predicates.CustomDataPredicate` -> `mc-math` (216.6s)
- [PASS] `net.minecraft.util.valueproviders.ClampedNormalInt` -> `mc-math` (212.5s)
- [PASS] `net.minecraft.core.GlobalPos` -> `mc-math` (205.3s)
- [PASS] `net.minecraft.core.ClientAsset` -> `mc-math` (205.3s)
