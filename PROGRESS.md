# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:35:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5818 / 66676 (8.726%)
- **Mapped Java Classes**: 1321 / 7163 (18.442%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2271 mapped methods
- `mc-worldgen`: 1488 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement` -> `mc-worldgen` (263.3s)
- [PASS] `net.minecraft.network.protocol.login.LoginProtocols` -> `mc-protocol` (567.3s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket` -> `mc-protocol` (567.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.parameters.LootContextParamSets` -> `mc-nbt` (838.5s)
