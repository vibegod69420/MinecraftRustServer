# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:54:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8096 / 66676 (12.142%)
- **Mapped Java Classes**: 1737 / 7163 (24.250%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2381 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2636 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (1331.3s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.FileIOStat` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TimedStatSummary` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.ChunkGenStat` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.StructureGenStat` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.world.level.chunk.ProtoChunk` -> `mc-worldgen` (1231.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V1486` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V4648` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V4656` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V2509` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.fixes.RenamedCoralFansFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.fixes.RecipesRenameningFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.fixes.FilteredBooksFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.schemas.V1451_5` -> `mc-math` (230.7s)
