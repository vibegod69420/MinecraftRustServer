# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:40:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5087 / 66676 (7.629%)
- **Mapped Java Classes**: 1187 / 7163 (16.571%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1786 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 1740 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (1330.7s)
- [PASS] `net.minecraft.util.ParticleUtils` -> `mc-math` (1399.4s)
- [PASS] `net.minecraft.world.level.storage.TagValueOutput` -> `mc-nbt` (954.3s)
- [FAIL] `net.minecraft.network.PacketEncoder` -> `mc-protocol` (1314.5s)
- [PASS] `net.minecraft.network.Utf8String` -> `mc-protocol` (1314.5s)
- [PASS] `net.minecraft.network.Varint21FrameDecoder` -> `mc-protocol` (1314.5s)
- [PASS] `net.minecraft.network.PacketBundlePacker` -> `mc-protocol` (1314.5s)
- [PASS] `net.minecraft.util.debug.DebugBrainDump` -> `mc-math` (702.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundContainerSetDataPacket` -> `mc-protocol` (728.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetScorePacket` -> `mc-protocol` (728.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTakeItemEntityPacket` -> `mc-protocol` (728.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRotateHeadPacket` -> `mc-protocol` (728.7s)
- [FAIL] `net.minecraft.util.FileUtil` -> `mc-math` (897.7s)
- [PASS] `net.minecraft.util.SpawnUtil` -> `mc-math` (1555.9s)
- [PASS] `net.minecraft.util.HttpUtil` -> `mc-math` (668.0s)
