# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:23:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7928 / 66676 (11.890%)
- **Mapped Java Classes**: 1671 / 7163 (23.328%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2181 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2601 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.schemas.V2704` -> `mc-math` (132.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V2571` -> `mc-math` (132.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V1931` -> `mc-math` (132.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V1904` -> `mc-math` (132.3s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (1823.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (633.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` -> `mc-worldgen` (2024.3s)
- [PASS] `net.minecraft.util.profiling.EmptyProfileResults` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.ResultField` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.ProfileResults` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.ProfileCollector` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.metrics.storage.RecordedDeviation` -> `mc-math` (100.8s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityShulkerBoxColorFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.FilteredSignsFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.DyeItemRenameFix` -> `mc-math` (183.2s)
