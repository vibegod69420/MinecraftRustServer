# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 21:36:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9652 / 66676 (14.476%)
- **Mapped Java Classes**: 2175 / 7163 (30.364%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1864 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (391.1s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (355.6s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (355.6s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (355.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (294.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (294.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (294.1s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (423.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (401.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (401.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (401.1s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (352.3s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (352.3s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (352.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (363.3s)
