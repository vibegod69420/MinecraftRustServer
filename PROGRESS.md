# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 02:24:02 UTC
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
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (1457.9s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (584.1s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (1150.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (960.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (950.6s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (950.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (729.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (685.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (629.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (572.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (512.1s)
- [PASS] `net.minecraft.network.chat.ComponentUtils` -> `mc-protocol` (490.3s)
- [FAIL] `net.minecraft.util.profiling.metrics.storage.MetricsPersister` -> `mc-math` (10682.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (10292.7s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (6456.7s)
