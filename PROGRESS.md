# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:42:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1536 / 66676 (2.304%)
- **Mapped Java Classes**: 451 / 7163 (6.296%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 606 mapped methods
- `mc-nbt`: 480 mapped methods
- `mc-protocol`: 309 mapped methods
- `mc-worldgen`: 446 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.ConstantValue` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.NumberProvider` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.LootItemFunction` -> `mc-nbt` (399.9s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.DiscardItem` -> `mc-nbt` (399.9s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.FunctionUserBuilder` -> `mc-nbt` (399.9s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.LootItemFunctionType` -> `mc-nbt` (399.9s)
- [PASS] `net.minecraft.core.component.predicates.EnchantmentsPredicate` -> `mc-math` (380.7s)
- [PASS] `net.minecraft.core.component.predicates.JukeboxPlayablePredicate` -> `mc-math` (380.7s)
- [PASS] `net.minecraft.core.component.predicates.FireworkExplosionPredicate` -> `mc-math` (380.7s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.InsideWorldBoundsPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.UnobstructedPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.NotPredicate` -> `mc-worldgen` (943.5s)
