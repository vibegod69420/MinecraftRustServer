# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 05:54:14 UTC
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
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (3186.8s)
- [FAIL] `net.minecraft.world.level.levelgen.blending.Blender` -> `mc-worldgen` (1773.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (1048.3s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (957.2s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (1193.1s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator` -> `mc-worldgen` (1634.1s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (584.3s)
- [FAIL] `net.minecraft.world.level.levelgen.carver.CaveWorldCarver` -> `mc-worldgen` (2778.6s)
- [FAIL] `net.minecraft.world.level.levelgen.Heightmap` -> `mc-worldgen` (2774.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (2501.0s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (2432.8s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1475.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.IcebergFeature` -> `mc-worldgen` (1078.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (1819.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (2247.4s)
