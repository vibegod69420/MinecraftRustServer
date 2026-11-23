# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 06:16:26 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7623 / 66676 (11.433%)
- **Mapped Java Classes**: 1611 / 7163 (22.491%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2328 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.Aquifer` -> `mc-worldgen` (1188.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFileStorage` -> `mc-worldgen` (479.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RecreatingSimpleRegionStorage` -> `mc-worldgen` (479.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.ChunkIOErrorReporter` -> `mc-worldgen` (479.7s)
- [PASS] `net.minecraft.world.level.chunk.LinearPalette` -> `mc-worldgen` (309.2s)
- [PASS] `net.minecraft.world.level.chunk.HashMapPalette` -> `mc-worldgen` (309.2s)
- [PASS] `net.minecraft.world.level.chunk.SingleValuePalette` -> `mc-worldgen` (309.2s)
- [PASS] `net.minecraft.world.level.chunk.status.ChunkDependencies` -> `mc-worldgen` (651.1s)
- [PASS] `net.minecraft.world.level.chunk.status.WorldGenContext` -> `mc-worldgen` (651.1s)
- [PASS] `net.minecraft.world.level.chunk.status.ChunkStatusTask` -> `mc-worldgen` (651.1s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (792.2s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (2042.6s)
- [PASS] `net.minecraft.world.level.chunk.StructureAccess` -> `mc-worldgen` (249.1s)
- [PASS] `net.minecraft.world.level.chunk.ChunkGenerators` -> `mc-worldgen` (249.1s)
- [PASS] `net.minecraft.world.level.chunk.Palette` -> `mc-worldgen` (249.1s)
