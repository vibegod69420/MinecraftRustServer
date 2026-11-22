# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:49:48 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2946 / 66676 (4.418%)
- **Mapped Java Classes**: 740 / 7163 (10.331%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1084 mapped methods
- `mc-nbt`: 988 mapped methods
- `mc-protocol`: 766 mapped methods
- `mc-worldgen`: 704 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator` -> `mc-nbt` (405.1s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.UniformGenerator` -> `mc-nbt` (405.1s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.NumberProviders` -> `mc-nbt` (405.1s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.StorageValue` -> `mc-nbt` (405.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecoratorType` -> `mc-worldgen` (665.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.CocoaDecorator` -> `mc-worldgen` (665.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLogsDecorator` -> `mc-worldgen` (665.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.TrunkVineDecorator` -> `mc-worldgen` (665.9s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (569.7s)
- [PASS] `net.minecraft.util.LinearCongruentialGenerator` -> `mc-math` (279.3s)
- [PASS] `net.minecraft.util.BitStorage` -> `mc-math` (279.3s)
- [PASS] `net.minecraft.util.SignatureUpdater` -> `mc-math` (279.3s)
- [PASS] `net.minecraft.util.ProgressListener` -> `mc-math` (279.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemCondition` -> `mc-nbt` (667.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition` -> `mc-nbt` (667.5s)
