# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 05:41:17 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7426 / 66676 (11.137%)
- **Mapped Java Classes**: 1582 / 7163 (22.086%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 2102 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` -> `mc-worldgen` (280.2s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionBitmap` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.LegacyTagFixer` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionStorageInfo` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.ChunkScanAccess` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.EndCityPieces` -> `mc-worldgen` (1448.3s)
- [PASS] `net.minecraft.world.level.chunk.LightChunkGetter` -> `mc-worldgen` (210.8s)
- [PASS] `net.minecraft.world.level.chunk.PaletteResize` -> `mc-worldgen` (210.8s)
- [PASS] `net.minecraft.world.level.chunk.MissingPaletteEntryException` -> `mc-worldgen` (210.8s)
- [PASS] `net.minecraft.world.level.chunk.BlockColumn` -> `mc-worldgen` (210.8s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator` -> `mc-worldgen` (1247.0s)
- [PASS] `net.minecraft.world.level.levelgen.blending.BlendingData` -> `mc-worldgen` (1235.0s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (2423.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.SpikeFeature` -> `mc-worldgen` (1946.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece` -> `mc-worldgen` (335.2s)
