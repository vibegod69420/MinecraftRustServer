# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:02:39 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4426 / 66676 (6.638%)
- **Mapped Java Classes**: 1083 / 7163 (15.119%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1563 mapped methods
- `mc-nbt`: 1252 mapped methods
- `mc-protocol`: 1483 mapped methods
- `mc-worldgen`: 1214 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.DarkOakFoliagePlacer` -> `mc-worldgen` (595.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.RandomSpreadFoliagePlacer` -> `mc-worldgen` (595.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.CopyBlockState` -> `mc-nbt` (543.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature` -> `mc-worldgen` (707.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (707.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BonusChestFeature` -> `mc-worldgen` (707.1s)
- [PASS] `net.minecraft.util.debug.DebugSubscriptions` -> `mc-math` (475.8s)
- [PASS] `net.minecraft.util.KeyframeTrackSampler` -> `mc-math` (659.7s)
- [PASS] `net.minecraft.util.CommonLinks` -> `mc-math` (659.7s)
- [PASS] `net.minecraft.util.StringUtil` -> `mc-math` (659.7s)
- [FAIL] `net.minecraft.util.DirectoryLock` -> `mc-math` (997.3s)
- [PASS] `net.minecraft.util.ThreadingDetector` -> `mc-math` (997.3s)
- [PASS] `net.minecraft.util.FileZipper` -> `mc-math` (997.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer` -> `mc-worldgen` (554.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier` -> `mc-worldgen` (554.6s)
