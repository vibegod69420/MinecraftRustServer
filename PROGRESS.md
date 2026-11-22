# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:36:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2427 / 66676 (3.640%)
- **Mapped Java Classes**: 618 / 7163 (8.628%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 947 mapped methods
- `mc-nbt`: 739 mapped methods
- `mc-protocol`: 612 mapped methods
- `mc-worldgen`: 594 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.ping.ServerboundPingRequestPacket` -> `mc-protocol` (80.3s)
- [FAIL] `net.minecraft.core.DefaultedMappedRegistry` -> `mc-math` (482.3s)
- [PASS] `net.minecraft.core.Cursor3D` -> `mc-math` (482.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration` -> `mc-worldgen` (178.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SpringConfiguration` -> `mc-worldgen` (178.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration` -> `mc-worldgen` (178.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.PointedDripstoneConfiguration` -> `mc-worldgen` (178.9s)
- [PASS] `net.minecraft.core.component.DataComponentGetter` -> `mc-math` (429.5s)
- [PASS] `net.minecraft.network.protocol.ProtocolCodecBuilder` -> `mc-protocol` (337.0s)
- [PASS] `net.minecraft.network.protocol.Packet` -> `mc-protocol` (337.0s)
- [PASS] `net.minecraft.nbt.ShortTag` -> `mc-nbt` (279.5s)
- [PASS] `net.minecraft.nbt.EndTag` -> `mc-nbt` (279.5s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (277.4s)
- [PASS] `net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket` -> `mc-protocol` (239.7s)
- [PASS] `net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket` -> `mc-protocol` (239.7s)
