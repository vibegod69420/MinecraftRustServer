# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:48:29 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8473 / 66676 (12.708%)
- **Mapped Java Classes**: 1881 / 7163 (26.260%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2912 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (1472.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V2684` -> `mc-math` (477.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V100` -> `mc-math` (477.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V3807` -> `mc-math` (477.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V3685` -> `mc-math` (477.6s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces` -> `mc-worldgen` (1472.3s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (1401.4s)
- [FAIL] `net.minecraft.util.debugchart.AbstractSampleLogger` -> `mc-math` (1363.7s)
- [FAIL] `net.minecraft.util.debugchart.LocalSampleLogger` -> `mc-math` (1363.7s)
- [PASS] `net.minecraft.util.debugchart.RemoteSampleLogger` -> `mc-math` (1363.7s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (1323.6s)
- [PASS] `net.minecraft.util.datafix.fixes.PlayerUUIDFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.PlayerRespawnDataFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.EmptyItemInVillagerTradeFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.AdvancementsRenameFix` -> `mc-math` (1243.0s)
