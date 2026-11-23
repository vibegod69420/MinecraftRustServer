# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 18:08:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9221 / 66676 (13.830%)
- **Mapped Java Classes**: 2114 / 7163 (29.513%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3976 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (3394.1s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (1095.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (286.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (286.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (286.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootDataType` -> `mc-nbt` (2804.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition` -> `mc-nbt` (395.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ConditionReference` -> `mc-nbt` (395.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LocationCheck` -> `mc-nbt` (395.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (933.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (933.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (933.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (833.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (833.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (833.6s)
