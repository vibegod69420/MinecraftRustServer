# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 01:36:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9513 / 66676 (14.268%)
- **Mapped Java Classes**: 2162 / 7163 (30.183%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4143 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3478 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (6456.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.NewVillageFix` -> `mc-math` (9346.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingItemFrameDirectionFix` -> `mc-math` (9346.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.BitStorageAlignFix` -> `mc-math` (8411.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.MobEffectIdFix` -> `mc-math` (449.8s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (9542.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (7734.5s)
- [FAIL] `net.minecraft.util.eventlog.EventLogDirectory` -> `mc-math` (4562.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (9345.2s)
- [FAIL] `net.minecraft.util.datafix.PackedBitStorage` -> `mc-math` (4065.7s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (4926.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingMotiveFix` -> `mc-math` (4096.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHealthFix` -> `mc-math` (4096.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (4096.0s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkBiomeFix` -> `mc-math` (4096.0s)
