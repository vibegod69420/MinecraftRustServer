# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 18:26:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9709 / 66676 (14.562%)
- **Mapped Java Classes**: 2194 / 7163 (30.630%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1941 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (762.7s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (718.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (478.3s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (4053.5s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (1760.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (691.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (341.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (850.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (540.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (355.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingItemFrameDirectionFix` -> `mc-math` (341.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.BitStorageAlignFix` -> `mc-math` (339.0s)
- [FAIL] `net.minecraft.util.eventlog.EventLogDirectory` -> `mc-math` (1901.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingItemFrameDirectionFix` -> `mc-math` (667.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (445.4s)
