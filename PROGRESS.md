# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:03:21 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 217 / 66676 (0.326%)
- **Mapped Java Classes**: 69 / 7163 (0.963%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 238 mapped methods
- `mc-nbt`: 8 mapped methods
- `mc-protocol`: 24 mapped methods
- `mc-worldgen`: 15 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.ping.ClientboundPongResponsePacket` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.PingPacketTypes` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.ClientPongPacketListener` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.ServerPingPacketListener` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.world.level.levelgen.material.WorldGenMaterialRule` -> `mc-worldgen` (163.9s)
- [PASS] `net.minecraft.world.level.levelgen.material.MaterialRuleList` -> `mc-worldgen` (163.9s)
- [PASS] `net.minecraft.network.codec.StreamDecoder` -> `mc-protocol` (121.3s)
- [PASS] `net.minecraft.util.random.WeightedRandom` -> `mc-math` (737.8s)
- [PASS] `net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload` -> `mc-protocol` (157.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructurePieceAccessor` -> `mc-worldgen` (189.0s)
- [PASS] `net.minecraft.network.codec.StreamEncoder` -> `mc-protocol` (477.7s)
- [PASS] `net.minecraft.network.codec.StreamMemberEncoder` -> `mc-protocol` (141.8s)
- [FAIL] `net.minecraft.nbt.NbtAccounterException` -> `mc-nbt` (363.3s)
- [FAIL] `net.minecraft.network.codec.StreamDecoder` -> `mc-protocol` (425.6s)
- [PASS] `net.minecraft.nbt.NbtFormatException` -> `mc-nbt` (360.4s)
