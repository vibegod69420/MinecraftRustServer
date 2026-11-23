# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:39:14 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5858 / 66676 (8.786%)
- **Mapped Java Classes**: 1330 / 7163 (18.568%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2314 mapped methods
- `mc-worldgen`: 1502 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (1308.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.GeodeConfiguration` -> `mc-worldgen` (129.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.ConcentricRingsStructurePlacement` -> `mc-worldgen` (148.4s)
- [FAIL] `net.minecraft.world.level.storage.LevelStorageSource` -> `mc-nbt` (969.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BambooFeature` -> `mc-worldgen` (860.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.UnderwaterMagmaFeature` -> `mc-worldgen` (860.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.IceSpikeFeature` -> `mc-worldgen` (249.1s)
- [PASS] `net.minecraft.world.level.levelgen.PhantomSpawner` -> `mc-worldgen` (356.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.FeatureCountTracker` -> `mc-worldgen` (546.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.FossilFeature` -> `mc-worldgen` (546.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces` -> `mc-worldgen` (332.6s)
- [FAIL] `net.minecraft.world.level.storage.TagValueInput` -> `mc-nbt` (871.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureType` -> `mc-worldgen` (298.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.BuiltInLootTables` -> `mc-nbt` (1011.3s)
- [FAIL] `net.minecraft.util.thread.ParallelMapTransform` -> `mc-math` (2545.8s)
