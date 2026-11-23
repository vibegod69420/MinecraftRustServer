# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:53:40 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6741 / 66676 (10.110%)
- **Mapped Java Classes**: 1462 / 7163 (20.410%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2992 mapped methods
- `mc-worldgen`: 1831 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.chat.ChatTypeDecoration` -> `mc-protocol` (165.3s)
- [PASS] `net.minecraft.world.level.levelgen.Column` -> `mc-worldgen` (283.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.FallenTreeFeature` -> `mc-worldgen` (620.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (1104.0s)
- [PASS] `net.minecraft.network.UnconfiguredPipelineHandler` -> `mc-protocol` (286.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces` -> `mc-worldgen` (588.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration` -> `mc-worldgen` (207.1s)
- [PASS] `net.minecraft.world.level.levelgen.FlatLevelSource` -> `mc-worldgen` (306.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket` -> `mc-protocol` (167.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundGameEventPacket` -> `mc-protocol` (327.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (961.0s)
- [PASS] `net.minecraft.network.chat.CommonComponents` -> `mc-protocol` (224.2s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (1522.4s)
- [FAIL] `net.minecraft.network.chat.contents.ScoreContents` -> `mc-protocol` (364.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureStart` -> `mc-worldgen` (269.4s)
