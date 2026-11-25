# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 05:38:05 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9675 / 66676 (14.511%)
- **Mapped Java Classes**: 2180 / 7163 (30.434%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1891 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (2247.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (1169.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (999.4s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (2021.6s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (909.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (1596.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (1323.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (1311.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (1306.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (1232.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (1034.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (945.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (928.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (867.4s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (532.3s)
