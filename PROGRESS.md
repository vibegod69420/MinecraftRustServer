# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:09:41 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 434 / 66676 (0.651%)
- **Mapped Java Classes**: 144 / 7163 (2.010%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 330 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 82 mapped methods
- `mc-worldgen`: 103 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (106.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureSpawnOverride` -> `mc-worldgen` (106.8s)
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
