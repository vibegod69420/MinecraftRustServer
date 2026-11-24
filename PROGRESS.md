# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 10:22:17 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9539 / 66676 (14.306%)
- **Mapped Java Classes**: 2164 / 7163 (30.211%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3493 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition` -> `mc-nbt` (267.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ConditionReference` -> `mc-nbt` (267.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LocationCheck` -> `mc-nbt` (267.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (263.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer` -> `mc-nbt` (263.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.TagEntry` -> `mc-nbt` (263.8s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (1480.3s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (581.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (659.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (659.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (659.1s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (364.5s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (364.5s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (364.5s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (287.9s)
