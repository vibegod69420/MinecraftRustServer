# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 14:50:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9577 / 66676 (14.364%)
- **Mapped Java Classes**: 2170 / 7163 (30.295%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1864 mapped methods
- `mc-protocol`: 3506 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (1562.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (1511.5s)
- [FAIL] `net.minecraft.util.profiling.metrics.storage.MetricsPersister` -> `mc-math` (1400.2s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (1328.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (1163.1s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (1076.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (959.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (870.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (845.8s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (792.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (459.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (431.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (352.7s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (2194.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingItemFrameDirectionFix` -> `mc-math` (652.7s)
