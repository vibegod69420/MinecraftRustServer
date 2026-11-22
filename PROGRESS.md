# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:18:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 688 / 66676 (1.032%)
- **Mapped Java Classes**: 226 / 7163 (3.155%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 419 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 155 mapped methods
- `mc-worldgen`: 214 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.particles.ShriekParticleOption` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.SculkChargeParticleOptions` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.SimpleParticleType` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.ScalableParticleOptionsBase` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.ParticleType` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.ParticleLimit` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.network.protocol.handshake.HandshakePacketTypes` -> `mc-protocol` (386.3s)
- [PASS] `net.minecraft.network.protocol.handshake.HandshakeProtocols` -> `mc-protocol` (386.3s)
- [PASS] `net.minecraft.network.protocol.handshake.ClientIntent` -> `mc-protocol` (386.3s)
- [PASS] `net.minecraft.network.protocol.handshake.ServerHandshakePacketListener` -> `mc-protocol` (386.3s)
- [PASS] `net.minecraft.world.level.levelgen.synth.NoiseUtils` -> `mc-math` (366.2s)
- [PASS] `net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise` -> `mc-math` (366.2s)
- [PASS] `net.minecraft.network.protocol.status.StatusPacketTypes` -> `mc-protocol` (286.4s)
- [PASS] `net.minecraft.network.protocol.status.ServerboundStatusRequestPacket` -> `mc-protocol` (286.4s)
- [PASS] `net.minecraft.network.protocol.status.ServerStatusPacketListener` -> `mc-protocol` (286.4s)
