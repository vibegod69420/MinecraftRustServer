# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:59:55 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8149 / 66676 (12.222%)
- **Mapped Java Classes**: 1759 / 7163 (24.557%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2440 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2636 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.OptionsForceVBOFix` -> `mc-math` (589.7s)
- [PASS] `net.minecraft.util.datafix.fixes.CatTypeFix` -> `mc-math` (589.7s)
- [PASS] `net.minecraft.util.datafix.fixes.TridentAnimationFix` -> `mc-math` (589.7s)
- [PASS] `net.minecraft.util.datafix.fixes.EntitySalmonSizeFix` -> `mc-math` (589.7s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (1581.0s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (1000.7s)
- [PASS] `net.minecraft.util.profiling.metrics.profiling.ProfilerSamplerAdapter` -> `mc-math` (172.5s)
- [PASS] `net.minecraft.util.profiling.metrics.profiling.InactiveMetricsRecorder` -> `mc-math` (172.5s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (1331.3s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.FileIOStat` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TimedStatSummary` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.ChunkGenStat` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.StructureGenStat` -> `mc-math` (264.8s)
- [PASS] `net.minecraft.world.level.chunk.ProtoChunk` -> `mc-worldgen` (1231.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V1486` -> `mc-math` (151.7s)
