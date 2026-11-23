# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 07:28:11 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8023 / 66676 (12.033%)
- **Mapped Java Classes**: 1714 / 7163 (23.928%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2330 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2601 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.profiling.jfr.event.ChunkRegionReadEvent` -> `mc-math` (184.7s)
- [PASS] `net.minecraft.util.profiling.jfr.event.PacketReceivedEvent` -> `mc-math` (184.7s)
- [PASS] `net.minecraft.util.profiling.jfr.event.PacketSentEvent` -> `mc-math` (184.7s)
- [PASS] `net.minecraft.util.profiling.jfr.event.WorldLoadFinishedEvent` -> `mc-math` (184.7s)
- [FAIL] `net.minecraft.world.level.chunk.ProtoChunk` -> `mc-worldgen` (2187.2s)
- [PASS] `net.minecraft.util.datafix.schemas.V1481` -> `mc-math` (151.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V1801` -> `mc-math` (151.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V4531` -> `mc-math` (151.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V2568` -> `mc-math` (151.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V4421` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3328` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V4543` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3799` -> `mc-math` (151.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V700` -> `mc-math` (408.2s)
- [PASS] `net.minecraft.util.datafix.schemas.V501` -> `mc-math` (408.2s)
