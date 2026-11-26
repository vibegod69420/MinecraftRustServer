# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 02:06:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9767 / 66676 (14.648%)
- **Mapped Java Classes**: 2208 / 7163 (30.825%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1969 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (1081.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (988.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (988.5s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (946.9s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (900.5s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (580.9s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (580.9s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (580.9s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (477.1s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (358.1s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.ConditionReference` -> `mc-nbt` (317.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (273.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (273.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (273.1s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (86.3s)
