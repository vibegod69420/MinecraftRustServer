# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 21:16:18 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9745 / 66676 (14.616%)
- **Mapped Java Classes**: 2203 / 7163 (30.755%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1941 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (389.1s)
- [FAIL] `net.minecraft.world.level.levelgen.blending.Blender` -> `mc-worldgen` (4501.3s)
- [FAIL] `net.minecraft.world.level.levelgen.carver.CaveWorldCarver` -> `mc-worldgen` (5216.0s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` -> `mc-worldgen` (3231.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (3746.5s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (2615.9s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (4052.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (3855.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (3738.3s)
- [FAIL] `net.minecraft.world.level.levelgen.Heightmap` -> `mc-worldgen` (3496.5s)
- [FAIL] `net.minecraft.util.profiling.metrics.storage.MetricsPersister` -> `mc-math` (1147.2s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (1193.0s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (2217.2s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (1118.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (1769.4s)
