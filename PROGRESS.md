# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 04:50:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9767 / 66676 (14.648%)
- **Mapped Java Classes**: 2208 / 7163 (30.825%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1969 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (1024.9s)
- [FAIL] `net.minecraft.util.parsing.packrat.Dictionary` -> `mc-math` (794.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.StructureSettingsFlattenFix` -> `mc-math` (707.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHealthFix` -> `mc-math` (707.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (707.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingMotiveFix` -> `mc-math` (707.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (610.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (602.0s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (4288.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (2168.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DeltaFeature` -> `mc-worldgen` (2168.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature` -> `mc-worldgen` (2168.8s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (1477.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (1439.3s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (1409.3s)
