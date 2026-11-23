# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 09:02:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8558 / 66676 (12.835%)
- **Mapped Java Classes**: 1910 / 7163 (26.665%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3031 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.util.datafix.schemas.V3689` -> `mc-math` (725.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V703` -> `mc-math` (725.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V3078` -> `mc-math` (725.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityFallDistanceFloatToDoubleFix` -> `mc-math` (391.3s)
