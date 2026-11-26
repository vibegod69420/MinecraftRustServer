# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 18:22:02 UTC
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
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (1360.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.CoralTreeFeature` -> `mc-worldgen` (1360.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SeaPickleFeature` -> `mc-worldgen` (1360.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SimpleBlockFeature` -> `mc-worldgen` (1360.7s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (1110.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (1092.0s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (1038.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (1038.1s)
- [FAIL] `net.minecraft.util.datafix.PackedBitStorage` -> `mc-math` (1007.0s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (987.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (962.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingMotiveFix` -> `mc-math` (962.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.StructureSettingsFlattenFix` -> `mc-math` (962.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHealthFix` -> `mc-math` (962.8s)
- [FAIL] `net.minecraft.util.parsing.packrat.Dictionary` -> `mc-math` (814.5s)
