# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:16:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4742 / 66676 (7.112%)
- **Mapped Java Classes**: 1128 / 7163 (15.748%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1681 mapped methods
- `mc-nbt`: 1403 mapped methods
- `mc-protocol`: 1534 mapped methods
- `mc-worldgen`: 1269 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.NoiseSettings` -> `mc-worldgen` (246.9s)
- [PASS] `net.minecraft.world.level.levelgen.NoiseRouter` -> `mc-worldgen` (246.9s)
- [PASS] `net.minecraft.world.level.levelgen.GeodeBlockSettings` -> `mc-worldgen` (246.9s)
- [PASS] `net.minecraft.world.level.storage.loot.LootPool` -> `mc-nbt` (379.7s)
- [PASS] `net.minecraft.util.SortedArraySet` -> `mc-math` (282.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider` -> `mc-worldgen` (677.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.DualNoiseProvider` -> `mc-worldgen` (677.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.NoiseThresholdProvider` -> `mc-worldgen` (677.2s)
- [PASS] `net.minecraft.util.BlockUtil` -> `mc-math` (398.2s)
- [FAIL] `net.minecraft.world.level.storage.ValueInputContextHelper` -> `mc-nbt` (617.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetAttributesFunction` -> `mc-nbt` (271.6s)
- [PASS] `net.minecraft.util.Ease` -> `mc-math` (278.7s)
- [PASS] `net.minecraft.util.thread.BlockableEventLoop` -> `mc-math` (391.9s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.TrapezoidHeight` -> `mc-worldgen` (304.5s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ExplorationMapFunction` -> `mc-nbt` (303.1s)
