# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 06:15:28 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7600 / 66676 (11.398%)
- **Mapped Java Classes**: 1608 / 7163 (22.449%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2303 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.chunk.status.ChunkDependencies` -> `mc-worldgen` (651.1s)
- [PASS] `net.minecraft.world.level.chunk.status.WorldGenContext` -> `mc-worldgen` (651.1s)
- [PASS] `net.minecraft.world.level.chunk.status.ChunkStatusTask` -> `mc-worldgen` (651.1s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (792.2s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (2042.6s)
- [PASS] `net.minecraft.world.level.chunk.StructureAccess` -> `mc-worldgen` (249.1s)
- [PASS] `net.minecraft.world.level.chunk.ChunkGenerators` -> `mc-worldgen` (249.1s)
- [PASS] `net.minecraft.world.level.chunk.Palette` -> `mc-worldgen` (249.1s)
- [PASS] `net.minecraft.world.level.chunk.LightChunk` -> `mc-worldgen` (249.1s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceSystem` -> `mc-worldgen` (1056.9s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator` -> `mc-worldgen` (811.8s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1147.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.IcebergFeature` -> `mc-worldgen` (837.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LargeDripstoneFeature` -> `mc-worldgen` (1028.0s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (1666.1s)
