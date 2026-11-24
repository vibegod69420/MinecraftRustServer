# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 10:34:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9544 / 66676 (14.314%)
- **Mapped Java Classes**: 2166 / 7163 (30.239%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3498 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (968.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (936.0s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (924.1s)
- [FAIL] `net.minecraft.nbt.TextComponentTagVisitor` -> `mc-nbt` (899.0s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (634.1s)
- [PASS] `net.minecraft.network.chat.SignableCommand` -> `mc-protocol` (435.2s)
- [FAIL] `net.minecraft.network.chat.contents.data.BlockDataSource` -> `mc-protocol` (405.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (385.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (385.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (385.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition` -> `mc-nbt` (267.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ConditionReference` -> `mc-nbt` (267.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LocationCheck` -> `mc-nbt` (267.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (263.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer` -> `mc-nbt` (263.8s)
