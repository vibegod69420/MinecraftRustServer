# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 01:18:01 UTC
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
- [FAIL] `net.minecraft.util.parsing.packrat.CachedParseState` -> `mc-math` (3668.3s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (4137.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (1298.8s)
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (966.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (717.3s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (3144.6s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (2916.1s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (2873.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkHeightAndBiomeFix` -> `mc-math` (408.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix` -> `mc-math` (828.5s)
- [FAIL] `net.minecraft.util.datafix.schemas.V705` -> `mc-math` (663.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (2831.2s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (1209.3s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (2607.6s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (2557.4s)
