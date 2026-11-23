# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:49:56 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4188 / 66676 (6.281%)
- **Mapped Java Classes**: 1022 / 7163 (14.268%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1491 mapped methods
- `mc-nbt`: 1200 mapped methods
- `mc-protocol`: 1369 mapped methods
- `mc-worldgen`: 1123 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.SpringFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ReplaceBlobsFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockPileFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.DiskFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration` -> `mc-worldgen` (439.7s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration` -> `mc-worldgen` (439.7s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CarverDebugSettings` -> `mc-worldgen` (439.7s)
- [FAIL] `net.minecraft.util.ByIdMap` -> `mc-math` (595.3s)
- [PASS] `net.minecraft.util.FormattedCharSequence` -> `mc-math` (595.3s)
- [PASS] `net.minecraft.util.InclusiveRange` -> `mc-math` (595.3s)
- [FAIL] `net.minecraft.util.EncoderCache` -> `mc-math` (922.3s)
- [PASS] `net.minecraft.util.DependencySorter` -> `mc-math` (922.3s)
- [PASS] `net.minecraft.util.PngInfo` -> `mc-math` (922.3s)
- [PASS] `net.minecraft.util.DelegateDataOutput` -> `mc-math` (922.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.EnchantWithLevelsFunction` -> `mc-nbt` (686.1s)
