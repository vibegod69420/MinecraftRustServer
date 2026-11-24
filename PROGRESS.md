# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 02:30:01 UTC
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
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (1283.0s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1116.8s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (888.7s)
- [FAIL] `net.minecraft.network.syncher.SynchedEntityData` -> `mc-protocol` (1809.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (1095.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (1663.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.IcebergFeature` -> `mc-worldgen` (468.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (642.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (1568.5s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (1547.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (1457.9s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (584.1s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (1150.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (960.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (950.6s)
