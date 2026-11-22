# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:30:34 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2272 / 66676 (3.408%)
- **Mapped Java Classes**: 588 / 7163 (8.209%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 938 mapped methods
- `mc-nbt`: 679 mapped methods
- `mc-protocol`: 528 mapped methods
- `mc-worldgen`: 569 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.ListTag` -> `mc-nbt` (181.4s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (123.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType` -> `mc-worldgen` (113.5s)
- [FAIL] `net.minecraft.core.Registry` -> `mc-math` (2076.1s)
- [FAIL] `net.minecraft.core.HolderSet` -> `mc-math` (1186.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition` -> `mc-nbt` (601.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemCondition` -> `mc-nbt` (601.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.MatchTool` -> `mc-nbt` (601.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck` -> `mc-nbt` (601.5s)
- [PASS] `net.minecraft.core.UUIDUtil` -> `mc-math` (1133.3s)
- [FAIL] `net.minecraft.nbt.NbtUtils` -> `mc-nbt` (2073.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.AllOfCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.ExplosionCondition` -> `mc-nbt` (805.3s)
