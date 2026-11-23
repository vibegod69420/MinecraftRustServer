# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:06:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6828 / 66676 (10.241%)
- **Mapped Java Classes**: 1474 / 7163 (20.578%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3014 mapped methods
- `mc-worldgen`: 1857 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (1068.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement` -> `mc-worldgen` (264.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (670.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundAddEntityPacket` -> `mc-protocol` (342.8s)
- [PASS] `net.minecraft.world.level.storage.TagValueInput` -> `mc-nbt` (855.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece` -> `mc-worldgen` (167.2s)
- [PASS] `net.minecraft.network.chat.ChatTypeDecoration` -> `mc-protocol` (165.3s)
- [PASS] `net.minecraft.world.level.levelgen.Column` -> `mc-worldgen` (283.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.FallenTreeFeature` -> `mc-worldgen` (620.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (1104.0s)
- [PASS] `net.minecraft.network.UnconfiguredPipelineHandler` -> `mc-protocol` (286.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces` -> `mc-worldgen` (588.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.TreeConfiguration` -> `mc-worldgen` (207.1s)
- [PASS] `net.minecraft.world.level.levelgen.FlatLevelSource` -> `mc-worldgen` (306.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket` -> `mc-protocol` (167.7s)
