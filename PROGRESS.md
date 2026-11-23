# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 05:50:14 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7470 / 66676 (11.203%)
- **Mapped Java Classes**: 1586 / 7163 (22.142%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3394 mapped methods
- `mc-worldgen`: 2136 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.RandomState` -> `mc-worldgen` (822.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (723.8s)
- [PASS] `net.minecraft.world.level.levelgen.Noises` -> `mc-worldgen` (644.8s)
- [FAIL] `net.minecraft.network.chat.ComponentUtils` -> `mc-protocol` (615.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket` -> `mc-protocol` (610.7s)
- [PASS] `net.minecraft.world.level.levelgen.structure.TemplateStructurePiece` -> `mc-worldgen` (578.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` -> `mc-worldgen` (280.2s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionBitmap` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.LegacyTagFixer` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionStorageInfo` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.ChunkScanAccess` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.EndCityPieces` -> `mc-worldgen` (1448.3s)
- [PASS] `net.minecraft.world.level.chunk.LightChunkGetter` -> `mc-worldgen` (210.8s)
- [PASS] `net.minecraft.world.level.chunk.PaletteResize` -> `mc-worldgen` (210.8s)
- [PASS] `net.minecraft.world.level.chunk.MissingPaletteEntryException` -> `mc-worldgen` (210.8s)
