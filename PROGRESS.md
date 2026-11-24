# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 02:42:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9539 / 66676 (14.306%)
- **Mapped Java Classes**: 2164 / 7163 (30.211%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3493 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (893.0s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (875.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (765.2s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (624.7s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (2439.5s)
- [FAIL] `net.minecraft.world.level.levelgen.Heightmap` -> `mc-worldgen` (1870.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (1677.1s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileStorage` -> `mc-worldgen` (315.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RecreatingSimpleRegionStorage` -> `mc-worldgen` (315.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.ChunkIOErrorReporter` -> `mc-worldgen` (315.6s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (1007.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (1929.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (1283.0s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1116.8s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (888.7s)
