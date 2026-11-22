# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:42:00 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2693 / 66676 (4.039%)
- **Mapped Java Classes**: 688 / 7163 (9.605%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1035 mapped methods
- `mc-nbt`: 845 mapped methods
- `mc-protocol`: 644 mapped methods
- `mc-worldgen`: 685 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.RegistrySetBuilder` -> `mc-math` (218.1s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.SlotLoot` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootPoolEntries` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootItem` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.DynamicLoot` -> `mc-nbt` (252.4s)
- [PASS] `net.minecraft.core.component.predicates.DataComponentPredicate` -> `mc-math` (660.2s)
- [PASS] `net.minecraft.nbt.NbtUtils` -> `mc-nbt` (759.7s)
- [PASS] `net.minecraft.core.cauldron.CauldronInteraction` -> `mc-math` (215.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureSet` -> `mc-worldgen` (257.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData` -> `mc-worldgen` (257.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.BuiltinStructureSets` -> `mc-worldgen` (257.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.SinglePieceStructure` -> `mc-worldgen` (257.3s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (688.9s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ContextScoreboardNameProvider` -> `mc-nbt` (395.8s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProviders` -> `mc-nbt` (395.8s)
