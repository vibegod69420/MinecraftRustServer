# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:22:37 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 811 / 66676 (1.216%)
- **Mapped Java Classes**: 271 / 7163 (3.783%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 497 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 176 mapped methods
- `mc-worldgen`: 245 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.valueproviders.ClampedNormalInt` -> `mc-math` (212.5s)
- [PASS] `net.minecraft.core.GlobalPos` -> `mc-math` (205.3s)
- [PASS] `net.minecraft.core.ClientAsset` -> `mc-math` (205.3s)
- [PASS] `net.minecraft.core.Rotations` -> `mc-math` (205.3s)
- [PASS] `net.minecraft.core.Direction8` -> `mc-math` (205.3s)
- [PASS] `net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket` -> `mc-protocol` (191.6s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket` -> `mc-protocol` (191.6s)
- [PASS] `net.minecraft.network.protocol.login.ClientLoginPacketListener` -> `mc-protocol` (191.6s)
- [PASS] `net.minecraft.network.protocol.login.ServerLoginPacketListener` -> `mc-protocol` (191.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement` -> `mc-worldgen` (149.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType` -> `mc-worldgen` (149.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.AboveRootPlacement` -> `mc-worldgen` (149.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.ReplaceablePredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.AllOfPredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.AnyOfPredicate` -> `mc-worldgen` (478.3s)
