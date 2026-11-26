# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 00:09:51 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9761 / 66676 (14.639%)
- **Mapped Java Classes**: 2207 / 7163 (30.811%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1963 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (252.0s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (252.0s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (100.9s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (31.7s)
- [FAIL] `net.minecraft.util.parsing.packrat.CachedParseState` -> `mc-math` (4004.3s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (4261.3s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (4116.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (2015.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (3041.3s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (3701.3s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (2090.2s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (1643.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (3195.3s)
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (1238.2s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunk` -> `mc-worldgen` (2984.2s)
