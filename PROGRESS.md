# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 15:22:12 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9791 / 66676 (14.684%)
- **Mapped Java Classes**: 2209 / 7163 (30.839%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1993 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (778.9s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (643.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (625.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (625.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (625.1s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (480.0s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (385.5s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (385.5s)
- [PASS] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (385.5s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (113.0s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (72.1s)
- [FAIL] `net.minecraft.util.parsing.packrat.CachedParseState` -> `mc-math` (5039.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (5183.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (1838.3s)
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (1347.4s)
