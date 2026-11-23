# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:34:24 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5815 / 66676 (8.721%)
- **Mapped Java Classes**: 1319 / 7163 (18.414%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2271 mapped methods
- `mc-worldgen`: 1483 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.BuriedTreasurePieces` -> `mc-worldgen` (332.6s)
- [FAIL] `net.minecraft.world.level.storage.TagValueInput` -> `mc-nbt` (871.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureType` -> `mc-worldgen` (298.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.BuiltInLootTables` -> `mc-nbt` (1011.3s)
- [FAIL] `net.minecraft.util.thread.ParallelMapTransform` -> `mc-math` (2545.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement` -> `mc-worldgen` (263.3s)
- [PASS] `net.minecraft.network.protocol.login.LoginProtocols` -> `mc-protocol` (567.3s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket` -> `mc-protocol` (567.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.parameters.LootContextParamSets` -> `mc-nbt` (838.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (626.9s)
- [FAIL] `net.minecraft.world.level.storage.ValueInputContextHelper` -> `mc-nbt` (742.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.ForkingTrunkPlacer` -> `mc-worldgen` (1092.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.BendingTrunkPlacer` -> `mc-worldgen` (1092.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetContainerContents` -> `mc-nbt` (1266.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator` -> `mc-worldgen` (276.4s)
