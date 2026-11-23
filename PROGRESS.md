# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 09:21:28 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8605 / 66676 (12.906%)
- **Mapped Java Classes**: 1929 / 7163 (26.930%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3107 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.schemas.V2501` -> `mc-math` (250.1s)
- [PASS] `net.minecraft.util.datafix.schemas.V4300` -> `mc-math` (250.1s)
- [PASS] `net.minecraft.util.datafix.schemas.V4301` -> `mc-math` (250.1s)
- [PASS] `net.minecraft.util.datafix.schemas.V1470` -> `mc-math` (250.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.MemoryExpiryDataFix` -> `mc-math` (765.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix` -> `mc-math` (765.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkTicketUnpackPosFix` -> `mc-math` (765.5s)
- [FAIL] `net.minecraft.util.datafix.fixes.AbstractBlockPropertyFix` -> `mc-math` (765.5s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityRenameFix` -> `mc-math` (770.2s)
- [PASS] `net.minecraft.util.datafix.fixes.CarvingStepRemoveFix` -> `mc-math` (770.2s)
- [PASS] `net.minecraft.util.datafix.fixes.LockComponentPredicateFix` -> `mc-math` (770.2s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityStringUuidFix` -> `mc-math` (770.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (2089.1s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (2089.1s)
- [PASS] `net.minecraft.util.datafix.schemas.V4307` -> `mc-math` (725.3s)
