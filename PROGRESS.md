# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:52:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6687 / 66676 (10.029%)
- **Mapped Java Classes**: 1454 / 7163 (20.299%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2984 mapped methods
- `mc-worldgen`: 1784 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration` -> `mc-worldgen` (207.1s)
- [PASS] `net.minecraft.world.level.levelgen.FlatLevelSource` -> `mc-worldgen` (306.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket` -> `mc-protocol` (167.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundGameEventPacket` -> `mc-protocol` (327.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (961.0s)
- [PASS] `net.minecraft.network.chat.CommonComponents` -> `mc-protocol` (224.2s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (1522.4s)
- [FAIL] `net.minecraft.network.chat.contents.ScoreContents` -> `mc-protocol` (364.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureStart` -> `mc-worldgen` (269.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor` -> `mc-worldgen` (747.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket` -> `mc-protocol` (246.2s)
- [PASS] `net.minecraft.network.protocol.status.ServerStatus` -> `mc-protocol` (176.8s)
- [PASS] `net.minecraft.util.HashOps` -> `mc-math` (1331.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FeatureCountTracker` -> `mc-worldgen` (887.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FossilFeature` -> `mc-worldgen` (887.4s)
