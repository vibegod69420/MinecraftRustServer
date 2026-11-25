# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 12:21:27 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9709 / 66676 (14.562%)
- **Mapped Java Classes**: 2194 / 7163 (30.630%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1941 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (626.5s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (352.4s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (1312.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition` -> `mc-nbt` (1262.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ConditionReference` -> `mc-nbt` (1262.7s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LocationCheck` -> `mc-nbt` (1262.7s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (1051.1s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (1014.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (751.5s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (751.5s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (751.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (741.0s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (518.8s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (1019.8s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (688.7s)
