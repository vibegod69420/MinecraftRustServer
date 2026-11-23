# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 15:56:41 UTC
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
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (5848.2s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (5544.8s)
- [FAIL] `net.minecraft.util.profiling.jfr.parse.JfrStatsParser` -> `mc-math` (5544.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.ParticleUnflatteningFix` -> `mc-math` (5544.8s)
- [FAIL] `net.minecraft.util.eventlog.EventLogDirectory` -> `mc-math` (5544.7s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (5544.2s)
- [FAIL] `net.minecraft.util.datafix.schemas.V704` -> `mc-math` (5544.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.SavedDataFeaturePoolElementFix` -> `mc-math` (5544.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (5544.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (2988.9s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemPotionFix` -> `mc-math` (1525.3s)
- [PASS] `net.minecraft.util.datafix.fixes.StructuresBecomeConfiguredFix` -> `mc-math` (1524.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (1524.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V1451_6` -> `mc-math` (1525.3s)
- [PASS] `net.minecraft.util.datafix.fixes.AdvancementsFix` -> `mc-math` (1525.2s)
