# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:06:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 356 / 66676 (0.534%)
- **Mapped Java Classes**: 117 / 7163 (1.633%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 288 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 59 mapped methods
- `mc-worldgen`: 60 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (364.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.PostPlacementProcessor` -> `mc-worldgen` (364.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.TerrainAdjustment` -> `mc-worldgen` (364.2s)
- [FAIL] `net.minecraft.network.chat.numbers.NumberFormat` -> `mc-protocol` (356.5s)
- [PASS] `net.minecraft.network.chat.numbers.NumberFormatType` -> `mc-protocol` (356.5s)
- [PASS] `net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.network.protocol.login.custom.DiscardedQueryPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.network.protocol.login.custom.CustomQueryPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.util.random.WeightedList` -> `mc-math` (314.8s)
- [PASS] `net.minecraft.network.LocalFrameEncoder` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.DisconnectionDetails` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.LocalFrameDecoder` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.ConnectionProtocol` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.ServerboundPacketListener` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.ClientboundPacketListener` -> `mc-protocol` (103.9s)
