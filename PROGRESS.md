# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 17:12:18 UTC
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
- [FAIL] `net.minecraft.util.datafix.fixes.NewVillageFix` -> `mc-math` (799.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.MemoryExpiryDataFix` -> `mc-math` (799.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix` -> `mc-math` (799.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces` -> `mc-worldgen` (707.3s)
- [FAIL] `net.minecraft.util.parsing.packrat.commands.Grammar` -> `mc-math` (649.6s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (3225.8s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (1584.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (1783.6s)
- [FAIL] `net.minecraft.util.parsing.packrat.CachedParseState` -> `mc-math` (2020.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (2666.4s)
- [FAIL] `net.minecraft.world.level.levelgen.carver.CaveWorldCarver` -> `mc-worldgen` (3222.2s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (1683.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.LevelDataGeneratorOptionsFix` -> `mc-math` (1059.7s)
- [FAIL] `net.minecraft.util.profiling.metrics.profiling.ActiveMetricsRecorder` -> `mc-math` (1286.6s)
- [FAIL] `net.minecraft.util.datafix.schemas.V704` -> `mc-math` (582.6s)
