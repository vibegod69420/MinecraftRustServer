# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 11:40:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9695 / 66676 (14.540%)
- **Mapped Java Classes**: 2188 / 7163 (30.546%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1915 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (475.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (475.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (475.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (387.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (387.4s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (387.4s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (341.8s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseChunk` -> `mc-worldgen` (2064.3s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (2013.9s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (1342.1s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (1342.1s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (1342.1s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (966.0s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (495.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (425.8s)
