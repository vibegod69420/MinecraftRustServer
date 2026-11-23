# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:47:49 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8038 / 66676 (12.055%)
- **Mapped Java Classes**: 1721 / 7163 (24.026%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2355 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2601 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.schemas.V1451_5` -> `mc-math` (230.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3682` -> `mc-math` (230.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3439_1` -> `mc-math` (230.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V701` -> `mc-math` (230.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (976.5s)
- [FAIL] `net.minecraft.world.level.chunk.storage.SerializableChunkData` -> `mc-worldgen` (425.2s)
- [FAIL] `net.minecraft.world.level.chunk.LevelChunkSection` -> `mc-worldgen` (779.0s)
- [FAIL] `net.minecraft.world.level.chunk.UpgradeData` -> `mc-worldgen` (477.9s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (472.5s)
- [PASS] `net.minecraft.util.profiling.jfr.event.ChunkRegionReadEvent` -> `mc-math` (184.7s)
- [PASS] `net.minecraft.util.profiling.jfr.event.PacketReceivedEvent` -> `mc-math` (184.7s)
- [PASS] `net.minecraft.util.profiling.jfr.event.PacketSentEvent` -> `mc-math` (184.7s)
- [PASS] `net.minecraft.util.profiling.jfr.event.WorldLoadFinishedEvent` -> `mc-math` (184.7s)
- [FAIL] `net.minecraft.world.level.chunk.ProtoChunk` -> `mc-worldgen` (2187.2s)
- [PASS] `net.minecraft.util.datafix.schemas.V1481` -> `mc-math` (151.6s)
