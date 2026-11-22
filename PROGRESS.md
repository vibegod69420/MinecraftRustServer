# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:14:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 520 / 66676 (0.780%)
- **Mapped Java Classes**: 171 / 7163 (2.387%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 375 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 99 mapped methods
- `mc-worldgen`: 155 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.valueproviders.TrapezoidFloat` -> `mc-math` (192.8s)
- [PASS] `net.minecraft.util.valueproviders.ClampedNormalFloat` -> `mc-math` (192.8s)
- [PASS] `net.minecraft.util.valueproviders.UniformFloat` -> `mc-math` (192.8s)
- [PASS] `net.minecraft.util.valueproviders.ClampedInt` -> `mc-math` (192.8s)
- [PASS] `net.minecraft.util.valueproviders.FloatProvider` -> `mc-math` (192.8s)
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
