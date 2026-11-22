# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:37:32 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2458 / 66676 (3.687%)
- **Mapped Java Classes**: 626 / 7163 (8.739%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 949 mapped methods
- `mc-nbt`: 739 mapped methods
- `mc-protocol`: 644 mapped methods
- `mc-worldgen`: 604 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.placement.HeightRangePlacement` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.PlacementContext` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.SurfaceWaterDepthFilter` -> `mc-worldgen` (598.4s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundTabListPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.core.particles.ParticleTypes` -> `mc-math` (184.9s)
- [PASS] `net.minecraft.network.protocol.ping.ServerboundPingRequestPacket` -> `mc-protocol` (80.3s)
- [FAIL] `net.minecraft.core.DefaultedMappedRegistry` -> `mc-math` (482.3s)
- [PASS] `net.minecraft.core.Cursor3D` -> `mc-math` (482.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SpikeConfiguration` -> `mc-worldgen` (178.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SpringConfiguration` -> `mc-worldgen` (178.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.DeltaFeatureConfiguration` -> `mc-worldgen` (178.9s)
