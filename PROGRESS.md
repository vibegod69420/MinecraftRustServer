# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 23:36:10 UTC
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
- [PASS] `net.minecraft.util.datafix.schemas.V2832` -> `mc-math` (441.2s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (2028.6s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (1927.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (1670.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (2296.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.DropChancesFormatFix` -> `mc-math` (562.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelUUIDFix` -> `mc-math` (562.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.FoodToConsumableFix` -> `mc-math` (562.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.StructureSettingsFlattenFix` -> `mc-math` (562.8s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (1158.0s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (2769.2s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.GcHeapStat` -> `mc-math` (836.1s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (1390.5s)
- [PASS] `net.minecraft.util.datafix.fixes.MapBannerBlockPosFormatFix` -> `mc-math` (785.5s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityAttributeBaseValueFix` -> `mc-math` (785.5s)
