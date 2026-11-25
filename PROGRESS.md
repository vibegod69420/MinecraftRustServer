# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 07:32:02 UTC
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
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (2499.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (348.8s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (1312.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.SavedDataFeaturePoolElementFix` -> `mc-math` (410.1s)
- [FAIL] `net.minecraft.util.parsing.packrat.commands.Grammar` -> `mc-math` (587.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (1008.5s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` -> `mc-worldgen` (2958.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (2214.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (3453.1s)
- [FAIL] `net.minecraft.world.level.levelgen.Heightmap` -> `mc-worldgen` (3182.4s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (748.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (3165.2s)
- [FAIL] `net.minecraft.world.level.chunk.UpgradeData` -> `mc-worldgen` (449.1s)
- [FAIL] `net.minecraft.world.level.levelgen.blending.Blender` -> `mc-worldgen` (2779.2s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (915.1s)
