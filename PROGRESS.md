# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:16:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 622 / 66676 (0.933%)
- **Mapped Java Classes**: 204 / 7163 (2.848%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 375 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 133 mapped methods
- `mc-worldgen`: 205 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.status.StatusPacketTypes` -> `mc-protocol` (286.4s)
- [PASS] `net.minecraft.network.protocol.status.ServerboundStatusRequestPacket` -> `mc-protocol` (286.4s)
- [PASS] `net.minecraft.network.protocol.status.ServerStatusPacketListener` -> `mc-protocol` (286.4s)
- [PASS] `net.minecraft.network.protocol.status.ClientStatusPacketListener` -> `mc-protocol` (286.4s)
- [FAIL] `net.minecraft.world.level.levelgen.synth.BlendedNoise` -> `mc-math` (254.7s)
- [PASS] `net.minecraft.world.level.levelgen.WorldGenerationContext` -> `mc-worldgen` (245.3s)
- [PASS] `net.minecraft.world.level.levelgen.GeodeLayerSettings` -> `mc-worldgen` (245.3s)
- [PASS] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (245.3s)
- [PASS] `net.minecraft.world.level.levelgen.GeodeCrackSettings` -> `mc-worldgen` (245.3s)
- [PASS] `net.minecraft.world.level.levelgen.WorldGenSettings` -> `mc-worldgen` (245.3s)
- [PASS] `net.minecraft.world.level.levelgen.PositionalRandomFactory` -> `mc-worldgen` (245.3s)
- [PASS] `net.minecraft.network.RegistryFriendlyByteBuf` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.BandwidthDebugMonitor` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.CipherDecoder` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.CipherEncoder` -> `mc-protocol` (213.3s)
