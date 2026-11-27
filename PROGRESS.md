# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-27 00:20:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9830 / 66676 (14.743%)
- **Mapped Java Classes**: 2211 / 7163 (30.867%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 2033 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (1908.1s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (1463.0s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (1390.2s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator` -> `mc-worldgen` (1118.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (1044.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.IcebergFeature` -> `mc-worldgen` (849.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileStorage` -> `mc-worldgen` (475.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RecreatingSimpleRegionStorage` -> `mc-worldgen` (475.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.ChunkIOErrorReporter` -> `mc-worldgen` (475.4s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (3959.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (1358.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (1093.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (975.0s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (809.9s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (780.3s)
