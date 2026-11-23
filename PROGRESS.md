# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 14:08:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9171 / 66676 (13.755%)
- **Mapped Java Classes**: 2106 / 7163 (29.401%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3909 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.profiling.metrics.storage.MetricsPersister` -> `mc-math` (3042.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (3042.0s)
- [FAIL] `net.minecraft.util.profiling.metrics.profiling.ActiveMetricsRecorder` -> `mc-math` (3041.8s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemSpawnEggFix` -> `mc-math` (3042.0s)
- [FAIL] `net.minecraft.util.datafix.schemas.V2832` -> `mc-math` (5438.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (3041.8s)
- [PASS] `net.minecraft.util.datafix.fixes.EntitySpawnerItemVariantComponentFix` -> `mc-math` (3041.6s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix` -> `mc-math` (4007.6s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityEquipmentToArmorAndHandFix` -> `mc-math` (1610.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.BitStorageAlignFix` -> `mc-math` (4007.5s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix` -> `mc-math` (4329.9s)
