# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:17:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 649 / 66676 (0.973%)
- **Mapped Java Classes**: 211 / 7163 (2.946%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 400 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 140 mapped methods
- `mc-worldgen`: 205 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.synth.NoiseUtils` -> `mc-math` (366.2s)
- [PASS] `net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise` -> `mc-math` (366.2s)
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
