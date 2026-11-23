# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:17:54 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3510 / 66676 (5.264%)
- **Mapped Java Classes**: 883 / 7163 (12.327%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1257 mapped methods
- `mc-nbt`: 1130 mapped methods
- `mc-protocol`: 993 mapped methods
- `mc-worldgen`: 872 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.EmptyPoolElement` -> `mc-worldgen` (238.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.LegacySinglePoolElement` -> `mc-worldgen` (238.3s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundSetBeaconPacket` -> `mc-protocol` (533.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket` -> `mc-protocol` (533.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundContainerClosePacket` -> `mc-protocol` (533.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetTestBlockPacket` -> `mc-protocol` (533.0s)
- [PASS] `net.minecraft.network.chat.contents.ObjectContents` -> `mc-protocol` (166.0s)
- [PASS] `net.minecraft.network.chat.numbers.NumberFormatTypes` -> `mc-protocol` (493.6s)
- [PASS] `net.minecraft.network.chat.numbers.FixedFormat` -> `mc-protocol` (493.6s)
- [PASS] `net.minecraft.network.chat.numbers.BlankFormat` -> `mc-protocol` (493.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (229.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (229.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (229.1s)
- [PASS] `net.minecraft.network.protocol.status.ClientboundStatusResponsePacket` -> `mc-protocol` (232.1s)
- [PASS] `net.minecraft.network.CompressionEncoder` -> `mc-protocol` (531.4s)
