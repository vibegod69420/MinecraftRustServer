# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:07:39 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 419 / 66676 (0.628%)
- **Mapped Java Classes**: 138 / 7163 (1.927%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 330 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 82 mapped methods
- `mc-worldgen`: 81 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.BundlePacket` -> `mc-protocol` (465.2s)
- [PASS] `net.minecraft.network.protocol.PacketFlow` -> `mc-protocol` (465.2s)
- [PASS] `net.minecraft.network.protocol.CodecModifier` -> `mc-protocol` (465.2s)
- [PASS] `net.minecraft.network.protocol.UnboundProtocol` -> `mc-protocol` (465.2s)
- [PASS] `net.minecraft.network.protocol.BundleDelimiterPacket` -> `mc-protocol` (465.2s)
- [PASS] `net.minecraft.network.protocol.SimpleUnboundProtocol` -> `mc-protocol` (465.2s)
- [PASS] `net.minecraft.world.level.levelgen.placement.InSquarePlacement` -> `mc-worldgen` (162.1s)
- [PASS] `net.minecraft.world.level.levelgen.placement.RarityFilter` -> `mc-worldgen` (162.1s)
- [PASS] `net.minecraft.world.level.levelgen.placement.CaveSurface` -> `mc-worldgen` (162.1s)
- [PASS] `net.minecraft.world.level.levelgen.placement.PlacementModifier` -> `mc-worldgen` (162.1s)
- [PASS] `net.minecraft.world.level.levelgen.placement.PlacementFilter` -> `mc-worldgen` (162.1s)
- [PASS] `net.minecraft.world.level.levelgen.placement.RepeatingPlacement` -> `mc-worldgen` (162.1s)
- [PASS] `net.minecraft.network.protocol.cookie.CookiePacketTypes` -> `mc-protocol` (121.2s)
- [PASS] `net.minecraft.network.protocol.cookie.ServerCookiePacketListener` -> `mc-protocol` (121.2s)
- [PASS] `net.minecraft.network.protocol.cookie.ClientCookiePacketListener` -> `mc-protocol` (121.2s)
