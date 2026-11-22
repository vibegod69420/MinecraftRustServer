# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:54:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1980 / 66676 (2.970%)
- **Mapped Java Classes**: 549 / 7163 (7.664%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 744 mapped methods
- `mc-nbt`: 594 mapped methods
- `mc-protocol`: 480 mapped methods
- `mc-worldgen`: 540 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.nbt.TextComponentTagVisitor` -> `mc-nbt` (1549.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundCooldownPacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetTimePacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRecipeBookRemovePacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.core.RegistryAccess` -> `mc-math` (258.4s)
- [FAIL] `net.minecraft.world.level.storage.ValueInput` -> `mc-nbt` (521.5s)
- [FAIL] `net.minecraft.world.level.storage.LevelResource` -> `mc-nbt` (521.5s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (521.5s)
- [PASS] `net.minecraft.world.level.storage.FileNameDateFormatter` -> `mc-nbt` (521.5s)
- [PASS] `net.minecraft.nbt.NbtOps` -> `mc-nbt` (1426.3s)
- [PASS] `net.minecraft.network.protocol.common.ClientCommonPacketListener` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundPongPacket` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundPingPacket` -> `mc-protocol` (197.7s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundTransferPacket` -> `mc-protocol` (197.7s)
