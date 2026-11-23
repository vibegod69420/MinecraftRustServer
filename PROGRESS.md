# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:53:55 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7292 / 66676 (10.937%)
- **Mapped Java Classes**: 1559 / 7163 (21.765%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 1952 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (992.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (1114.8s)
- [FAIL] `net.minecraft.server.ServerTickRateManager` -> `mc-protocol` (1061.4s)
- [FAIL] `net.minecraft.world.level.levelgen.carver.CaveWorldCarver` -> `mc-worldgen` (1332.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (871.5s)
