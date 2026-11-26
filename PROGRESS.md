# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 07:24:02 UTC
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
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (4238.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (735.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (279.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingItemFrameDirectionFix` -> `mc-math` (1452.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.DropChancesFormatFix` -> `mc-math` (1452.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelUUIDFix` -> `mc-math` (1452.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.FoodToConsumableFix` -> `mc-math` (1452.4s)
- [FAIL] `net.minecraft.util.eventlog.EventLogDirectory` -> `mc-math` (1422.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.BitStorageAlignFix` -> `mc-math` (1363.4s)
- [FAIL] `net.minecraft.util.datafix.PackedBitStorage` -> `mc-math` (1231.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (1024.9s)
- [FAIL] `net.minecraft.util.parsing.packrat.Dictionary` -> `mc-math` (794.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.StructureSettingsFlattenFix` -> `mc-math` (707.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHealthFix` -> `mc-math` (707.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (707.5s)
