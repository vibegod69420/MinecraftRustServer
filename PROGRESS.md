# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 09:12:02 UTC
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
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (368.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (368.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (368.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (336.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (336.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (336.2s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (322.4s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (248.3s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (248.3s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (248.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (377.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (377.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (377.6s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (92.8s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (98.5s)
