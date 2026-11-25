# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 20:34:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9709 / 66676 (14.562%)
- **Mapped Java Classes**: 2194 / 7163 (30.630%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1941 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (3738.3s)
- [FAIL] `net.minecraft.world.level.levelgen.Heightmap` -> `mc-worldgen` (3496.5s)
- [FAIL] `net.minecraft.util.profiling.metrics.storage.MetricsPersister` -> `mc-math` (1147.2s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (1193.0s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (2217.2s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (1118.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (1769.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileStorage` -> `mc-worldgen` (752.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RecreatingSimpleRegionStorage` -> `mc-worldgen` (752.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.ChunkIOErrorReporter` -> `mc-worldgen` (752.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.IcebergFeature` -> `mc-worldgen` (1477.6s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator` -> `mc-worldgen` (1149.0s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (2574.0s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1374.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (946.1s)
