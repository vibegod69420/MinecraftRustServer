# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 12:01:24 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9701 / 66676 (14.550%)
- **Mapped Java Classes**: 2192 / 7163 (30.602%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1928 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (751.5s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (751.5s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (751.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (741.0s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (518.8s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (1019.8s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (688.7s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (532.8s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (532.8s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (532.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (475.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (475.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (475.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (387.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (387.4s)
