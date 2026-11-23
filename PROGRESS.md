# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:42:22 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6461 / 66676 (9.690%)
- **Mapped Java Classes**: 1417 / 7163 (19.782%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2824 mapped methods
- `mc-worldgen`: 1731 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.BambooFeature` -> `mc-worldgen` (1166.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.EndPodiumFeature` -> `mc-worldgen` (1166.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (641.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (543.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket` -> `mc-protocol` (701.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket` -> `mc-protocol` (701.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (1146.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure` -> `mc-worldgen` (1146.2s)
- [PASS] `net.minecraft.network.chat.LastSeenMessagesValidator` -> `mc-protocol` (375.0s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (984.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDamageEventPacket` -> `mc-protocol` (915.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket` -> `mc-protocol` (915.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket` -> `mc-protocol` (915.2s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (832.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BasaltPillarFeature` -> `mc-worldgen` (322.2s)
