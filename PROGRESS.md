# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:38:48 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2504 / 66676 (3.756%)
- **Mapped Java Classes**: 637 / 7163 (8.893%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 949 mapped methods
- `mc-nbt`: 766 mapped methods
- `mc-protocol`: 644 mapped methods
- `mc-worldgen`: 630 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider` -> `mc-nbt` (395.8s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProviders` -> `mc-nbt` (395.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.LavaSubmergedBlockProcessor` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.LinearPosTest` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.RuleProcessor` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.ProtectedBlockProcessor` -> `mc-worldgen` (648.3s)
- [PASS] `net.minecraft.world.level.levelgen.placement.HeightRangePlacement` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.NoiseThresholdCountPlacement` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.PlacementContext` -> `mc-worldgen` (598.4s)
- [PASS] `net.minecraft.world.level.levelgen.placement.SurfaceWaterDepthFilter` -> `mc-worldgen` (598.4s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundTabListPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundContainerButtonClickPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket` -> `mc-protocol` (547.2s)
- [PASS] `net.minecraft.core.particles.ParticleTypes` -> `mc-math` (184.9s)
