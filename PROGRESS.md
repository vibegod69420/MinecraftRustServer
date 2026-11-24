# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 11:16:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9577 / 66676 (14.364%)
- **Mapped Java Classes**: 2170 / 7163 (30.295%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1864 mapped methods
- `mc-protocol`: 3506 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.SeaPickleFeature` -> `mc-worldgen` (2010.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SimpleBlockFeature` -> `mc-worldgen` (2010.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (2010.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.CoralTreeFeature` -> `mc-worldgen` (2010.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (1178.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (1052.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DeltaFeature` -> `mc-worldgen` (1052.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature` -> `mc-worldgen` (1052.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (935.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (935.5s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (913.9s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (913.5s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (809.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (670.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (670.7s)
