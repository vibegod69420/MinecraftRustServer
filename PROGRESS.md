# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 05:01:22 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7317 / 66676 (10.974%)
- **Mapped Java Classes**: 1561 / 7163 (21.793%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 1980 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer` -> `mc-worldgen` (1540.6s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (2020.7s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets` -> `mc-worldgen` (853.1s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces` -> `mc-worldgen` (180.4s)
