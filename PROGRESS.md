# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 05:11:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7384 / 66676 (11.074%)
- **Mapped Java Classes**: 1568 / 7163 (21.890%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 2048 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (2423.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.SpikeFeature` -> `mc-worldgen` (1946.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece` -> `mc-worldgen` (335.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.LegacyStructureDataHandler` -> `mc-worldgen` (1491.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.IcebergFeature` -> `mc-worldgen` (974.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (1001.0s)
- [FAIL] `net.minecraft.world.level.levelgen.Heightmap` -> `mc-worldgen` (2327.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece` -> `mc-worldgen` (640.5s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorSettings` -> `mc-worldgen` (1079.1s)
- [PASS] `net.minecraft.world.level.levelgen.Beardifier` -> `mc-worldgen` (1859.0s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1154.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (923.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (1759.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (760.3s)
- [PASS] `net.minecraft.world.level.storage.LevelStorageSource` -> `mc-nbt` (1838.4s)
