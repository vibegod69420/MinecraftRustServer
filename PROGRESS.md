# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 12:34:04 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9036 / 66676 (13.552%)
- **Mapped Java Classes**: 2068 / 7163 (28.871%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3694 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.profiling.metrics.MetricsRegistry` -> `mc-math` (6999.2s)
- [PASS] `net.minecraft.util.datafix.fixes.NewVillageFix` -> `mc-math` (2213.7s)
- [PASS] `net.minecraft.util.datafix.fixes.SaddleEquipmentSlotFix` -> `mc-math` (2213.7s)
- [PASS] `net.minecraft.util.datafix.fixes.BoatSplitFix` -> `mc-math` (2213.7s)
- [PASS] `net.minecraft.util.datafix.fixes.OminousBannerRarityFix` -> `mc-math` (2213.7s)
- [FAIL] `net.minecraft.util.parsing.packrat.Dictionary` -> `mc-math` (3466.5s)
- [PASS] `net.minecraft.util.datafix.fixes.BannerEntityCustomNameToOverrideComponentFix` -> `mc-math` (7891.5s)
- [PASS] `net.minecraft.util.datafix.fixes.MobSpawnerEntityIdentifiersFix` -> `mc-math` (7891.5s)
- [PASS] `net.minecraft.util.datafix.fixes.AttributesRenameLegacy` -> `mc-math` (7891.5s)
- [PASS] `net.minecraft.util.profiling.jfr.parse.JfrStatsResult` -> `mc-math` (7891.1s)
- [PASS] `net.minecraft.util.datafix.fixes.FixProjectileStoredItem` -> `mc-math` (2213.6s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackCustomNameToOverrideComponentFix` -> `mc-math` (2213.6s)
- [PASS] `net.minecraft.util.datafix.fixes.RemoveBlockEntityTagFix` -> `mc-math` (2213.5s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityTheRenameningFix` -> `mc-math` (2213.5s)
- [PASS] `net.minecraft.util.datafix.schemas.V2551` -> `mc-math` (6998.9s)
