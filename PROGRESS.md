# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:46:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6582 / 66676 (9.872%)
- **Mapped Java Classes**: 1437 / 7163 (20.061%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2919 mapped methods
- `mc-worldgen`: 1737 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket` -> `mc-protocol` (246.2s)
- [PASS] `net.minecraft.network.protocol.status.ServerStatus` -> `mc-protocol` (176.8s)
- [PASS] `net.minecraft.util.HashOps` -> `mc-math` (1331.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FeatureCountTracker` -> `mc-worldgen` (887.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FossilFeature` -> `mc-worldgen` (887.4s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` -> `mc-protocol` (595.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundUpdateMobEffectPacket` -> `mc-protocol` (595.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket` -> `mc-protocol` (776.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket` -> `mc-protocol` (776.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BambooFeature` -> `mc-worldgen` (1166.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.EndPodiumFeature` -> `mc-worldgen` (1166.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (641.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (543.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket` -> `mc-protocol` (701.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket` -> `mc-protocol` (701.2s)
