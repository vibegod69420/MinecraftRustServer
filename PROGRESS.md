# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:53:28 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1961 / 66676 (2.941%)
- **Mapped Java Classes**: 545 / 7163 (7.609%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 744 mapped methods
- `mc-nbt`: 594 mapped methods
- `mc-protocol`: 461 mapped methods
- `mc-worldgen`: 540 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.NbtOps` -> `mc-nbt` (1426.3s)
- [PASS] `net.minecraft.network.protocol.common.ClientCommonPacketListener` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundPongPacket` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundPingPacket` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundTransferPacket` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SculkPatchConfiguration` -> `mc-worldgen` (201.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.RandomFeatureConfiguration` -> `mc-worldgen` (201.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.RandomBooleanFeatureConfiguration` -> `mc-worldgen` (201.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.EndGatewayConfiguration` -> `mc-worldgen` (201.5s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetPotionFunction` -> `mc-nbt` (218.2s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetComponentsFunction` -> `mc-nbt` (218.2s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.LimitCount` -> `mc-nbt` (218.2s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetItemFunction` -> `mc-nbt` (218.2s)
- [PASS] `net.minecraft.core.component.predicates.DataComponentPredicates` -> `mc-math` (363.7s)
- [PASS] `net.minecraft.core.component.predicates.WrittenBookPredicate` -> `mc-math` (363.7s)
