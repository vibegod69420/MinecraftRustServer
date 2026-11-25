# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-25 09:34:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9675 / 66676 (14.511%)
- **Mapped Java Classes**: 2180 / 7163 (30.434%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1891 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkStructuresTemplateRenameFix` -> `mc-math` (345.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (848.0s)
- [FAIL] `net.minecraft.world.level.chunk.UpgradeData` -> `mc-worldgen` (736.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.TextComponentHoverAndClickEventFix` -> `mc-math` (599.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.NewVillageFix` -> `mc-math` (660.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix` -> `mc-math` (660.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.MemoryExpiryDataFix` -> `mc-math` (660.5s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (576.2s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (117.8s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (51.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (5254.3s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (5914.3s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (2946.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkProtoTickListFix` -> `mc-math` (3361.4s)
- [FAIL] `net.minecraft.util.profiling.jfr.JfrProfiler` -> `mc-math` (3203.1s)
