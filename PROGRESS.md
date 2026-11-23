# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 18:35:20 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9345 / 66676 (14.015%)
- **Mapped Java Classes**: 2133 / 7163 (29.778%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4072 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3450 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.thread.ParallelMapTransform` -> `mc-math` (8382.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (305.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer` -> `mc-nbt` (305.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.TagEntry` -> `mc-nbt` (305.0s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (8380.4s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (8092.8s)
- [FAIL] `net.minecraft.core.MappedRegistry` -> `mc-math` (8079.4s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (3394.1s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (1095.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (286.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (286.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (286.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootDataType` -> `mc-nbt` (2804.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition` -> `mc-nbt` (395.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ConditionReference` -> `mc-nbt` (395.5s)
