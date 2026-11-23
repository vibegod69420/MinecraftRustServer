# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:20:17 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7919 / 66676 (11.877%)
- **Mapped Java Classes**: 1666 / 7163 (23.258%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2172 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2601 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.pools.JigsawPlacement` -> `mc-worldgen` (2024.3s)
- [PASS] `net.minecraft.util.profiling.EmptyProfileResults` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.ResultField` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.ProfileResults` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.ProfileCollector` -> `mc-math` (178.5s)
- [PASS] `net.minecraft.util.profiling.metrics.storage.RecordedDeviation` -> `mc-math` (100.8s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityShulkerBoxColorFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.FilteredSignsFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.DyeItemRenameFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityTippedArrowFix` -> `mc-math` (183.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (1820.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (1781.7s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (1720.3s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (1578.8s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (1004.0s)
