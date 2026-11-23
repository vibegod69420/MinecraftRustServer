# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:52:11 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8089 / 66676 (12.132%)
- **Mapped Java Classes**: 1734 / 7163 (24.208%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2374 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2636 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.chunk.ProtoChunk` -> `mc-worldgen` (1231.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V1486` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V4648` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V4656` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V2509` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.fixes.RenamedCoralFansFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.fixes.RecipesRenameningFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityElderGuardianSplitFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.fixes.FilteredBooksFix` -> `mc-math` (181.1s)
- [PASS] `net.minecraft.util.datafix.schemas.V1451_5` -> `mc-math` (230.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3682` -> `mc-math` (230.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3439_1` -> `mc-math` (230.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V701` -> `mc-math` (230.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (976.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (425.2s)
