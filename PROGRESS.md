# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:54:04 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8499 / 66676 (12.747%)
- **Mapped Java Classes**: 1891 / 7163 (26.400%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2941 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.fixes.EntityFallDistanceFloatToDoubleFix` -> `mc-math` (391.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityMinecartIdentifiersFix` -> `mc-math` (391.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.BlockPropertyRenameAndFix` -> `mc-math` (391.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix` -> `mc-math` (391.3s)
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
