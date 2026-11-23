# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 22:12:16 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9484 / 66676 (14.224%)
- **Mapped Java Classes**: 2153 / 7163 (30.057%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4092 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3478 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.parameters.LootContextParamSets` -> `mc-nbt` (567.6s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (1407.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (543.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (1320.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.configurations.MultifaceGrowthConfiguration` -> `mc-worldgen` (447.6s)
- [PASS] `net.minecraft.world.level.storage.PlayerDataStorage` -> `mc-nbt` (324.2s)
- [PASS] `net.minecraft.world.level.storage.LevelData` -> `mc-nbt` (324.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (4362.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.CoralTreeFeature` -> `mc-worldgen` (4362.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SeaPickleFeature` -> `mc-worldgen` (4362.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SimpleBlockFeature` -> `mc-worldgen` (4362.1s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetContainerContents` -> `mc-nbt` (192.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (998.3s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (3752.3s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (4210.5s)
