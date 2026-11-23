# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:56:19 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5194 / 66676 (7.790%)
- **Mapped Java Classes**: 1211 / 7163 (16.906%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1834 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 1845 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatPacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.status.StatusProtocols` -> `mc-protocol` (162.1s)
- [PASS] `net.minecraft.network.chat.numbers.StyledFormat` -> `mc-protocol` (148.3s)
- [FAIL] `net.minecraft.world.level.storage.DimensionDataStorage` -> `mc-nbt` (2089.5s)
- [PASS] `net.minecraft.util.ProblemReporter` -> `mc-math` (1434.7s)
- [FAIL] `net.minecraft.util.thread.ParallelMapTransform` -> `mc-math` (1454.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (1330.7s)
- [PASS] `net.minecraft.util.ParticleUtils` -> `mc-math` (1399.4s)
- [PASS] `net.minecraft.world.level.storage.TagValueOutput` -> `mc-nbt` (954.3s)
- [FAIL] `net.minecraft.network.PacketEncoder` -> `mc-protocol` (1314.5s)
- [PASS] `net.minecraft.network.Utf8String` -> `mc-protocol` (1314.5s)
- [PASS] `net.minecraft.network.Varint21FrameDecoder` -> `mc-protocol` (1314.5s)
