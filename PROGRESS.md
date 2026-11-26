# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 17:26:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9830 / 66676 (14.743%)
- **Mapped Java Classes**: 2211 / 7163 (30.867%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 2033 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.EntityIdFix` -> `mc-math` (758.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsHeightAndBiomeFix` -> `mc-math` (826.8s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (1205.9s)
- [PASS] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (1205.9s)
- [FAIL] `net.minecraft.util.parsing.packrat.Dictionary` -> `mc-math` (1026.1s)
- [FAIL] `net.minecraft.util.datafix.PackedBitStorage` -> `mc-math` (1096.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHealthFix` -> `mc-math` (1106.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (1106.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingMotiveFix` -> `mc-math` (1106.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.StructureSettingsFlattenFix` -> `mc-math` (1106.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (1028.2s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (1000.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.DropChancesFormatFix` -> `mc-math` (988.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelUUIDFix` -> `mc-math` (988.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.FoodToConsumableFix` -> `mc-math` (988.7s)
