# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 15:39:37 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9221 / 66676 (13.830%)
- **Mapped Java Classes**: 2114 / 7163 (29.513%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3976 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (2988.9s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemPotionFix` -> `mc-math` (1525.3s)
- [PASS] `net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix` -> `mc-math` (1524.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (1524.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V1451_6` -> `mc-math` (1525.3s)
- [PASS] `net.minecraft.util.datafix.fixes.AdvancementsFix` -> `mc-math` (1525.2s)
- [FAIL] `net.minecraft.util.parsing.packrat.CachedParseState` -> `mc-math` (1525.2s)
- [PASS] `net.minecraft.util.datafix.fixes.EquipmentFormatFix` -> `mc-math` (482.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.TextComponentHoverAndClickEventFix` -> `mc-math` (1546.2s)
- [FAIL] `net.minecraft.util.profiling.metrics.storage.MetricsPersister` -> `mc-math` (3042.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (3042.0s)
- [FAIL] `net.minecraft.util.profiling.metrics.profiling.ActiveMetricsRecorder` -> `mc-math` (3041.8s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemSpawnEggFix` -> `mc-math` (3042.0s)
- [FAIL] `net.minecraft.util.datafix.schemas.V2832` -> `mc-math` (5438.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (3041.8s)
