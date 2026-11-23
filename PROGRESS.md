# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 06:36:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7814 / 66676 (11.719%)
- **Mapped Java Classes**: 1635 / 7163 (22.826%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2555 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (836.0s)
- [PASS] `net.minecraft.world.level.chunk.storage.SimpleRegionStorage` -> `mc-worldgen` (1089.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.EntityStorage` -> `mc-worldgen` (687.4s)
- [PASS] `net.minecraft.world.level.chunk.ImposterProtoChunk` -> `mc-worldgen` (615.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.OceanRuinPieces` -> `mc-worldgen` (2062.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileVersion` -> `mc-worldgen` (610.1s)
- [PASS] `net.minecraft.world.level.chunk.DataLayer` -> `mc-worldgen` (646.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.JungleTemplePiece` -> `mc-worldgen` (1534.3s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkSource` -> `mc-worldgen` (742.4s)
- [PASS] `net.minecraft.world.level.chunk.BulkSectionAccess` -> `mc-worldgen` (742.4s)
- [PASS] `net.minecraft.world.level.chunk.EmptyLevelChunk` -> `mc-worldgen` (742.4s)
- [PASS] `net.minecraft.world.level.chunk.PalettedContainerFactory` -> `mc-worldgen` (742.4s)
- [PASS] `net.minecraft.world.level.chunk.status.ChunkStatus` -> `mc-worldgen` (265.5s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` -> `mc-worldgen` (1235.6s)
- [PASS] `net.minecraft.world.level.chunk.Strategy` -> `mc-worldgen` (303.4s)
