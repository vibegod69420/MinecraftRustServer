# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 20:10:01 UTC
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
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1374.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (946.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (2099.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (1875.5s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (1584.6s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (1522.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (1471.7s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (1450.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (1306.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (1180.7s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (1165.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (955.9s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (366.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (565.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (548.3s)
