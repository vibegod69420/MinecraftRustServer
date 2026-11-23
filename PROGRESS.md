# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:06:19 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5362 / 66676 (8.042%)
- **Mapped Java Classes**: 1254 / 7163 (17.507%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1901 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 1983 mapped methods
- `mc-worldgen`: 1298 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.worldupdate.WorldUpgrader` -> `mc-math` (310.4s)
- [FAIL] `net.minecraft.util.FastBufferedInputStream` -> `mc-math` (780.7s)
- [FAIL] `net.minecraft.util.SequencedPriorityIterator` -> `mc-math` (780.7s)
- [PASS] `net.minecraft.util.ByIdMap` -> `mc-math` (780.7s)
- [PASS] `net.minecraft.util.EncoderCache` -> `mc-math` (780.7s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (250.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.DripstoneClusterConfiguration` -> `mc-worldgen` (172.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.VegetationPatchConfiguration` -> `mc-worldgen` (172.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator` -> `mc-worldgen` (558.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (558.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (558.7s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (524.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece` -> `mc-worldgen` (299.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.BuiltinStructures` -> `mc-worldgen` (299.4s)
- [PASS] `net.minecraft.util.DirectoryLock` -> `mc-math` (439.9s)
