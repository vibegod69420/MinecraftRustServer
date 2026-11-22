# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:14:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 467 / 66676 (0.700%)
- **Mapped Java Classes**: 159 / 7163 (2.220%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 334 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 99 mapped methods
- `mc-worldgen`: 121 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.chat.numbers.NumberFormat` -> `mc-protocol` (122.1s)
- [PASS] `net.minecraft.network.chat.contents.objects.ObjectInfos` -> `mc-protocol` (143.9s)
- [PASS] `net.minecraft.network.chat.contents.objects.ObjectInfo` -> `mc-protocol` (143.9s)
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
