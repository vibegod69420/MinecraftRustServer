# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:18:18 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7889 / 66676 (11.832%)
- **Mapped Java Classes**: 1661 / 7163 (23.189%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2123 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2601 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityShulkerBoxColorFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.FilteredSignsFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.DyeItemRenameFix` -> `mc-math` (183.2s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityTippedArrowFix` -> `mc-math` (183.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (1820.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.MineshaftPieces` -> `mc-worldgen` (1781.7s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (1720.3s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (1578.8s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (1004.0s)
- [PASS] `net.minecraft.world.level.chunk.storage.EntityStorage` -> `mc-worldgen` (1375.7s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (1136.6s)
- [PASS] `net.minecraft.world.level.chunk.ChunkSource` -> `mc-worldgen` (526.3s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (433.5s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionFileVersion` -> `mc-worldgen` (269.0s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileStorage` -> `mc-worldgen` (219.5s)
