# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 05:05:24 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7327 / 66676 (10.989%)
- **Mapped Java Classes**: 1562 / 7163 (21.806%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 1990 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.network.syncher.SynchedEntityData` -> `mc-protocol` (2650.6s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (864.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.StructurePieceType` -> `mc-worldgen` (621.9s)
