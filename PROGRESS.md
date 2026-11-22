# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:31:48 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1224 / 66676 (1.836%)
- **Mapped Java Classes**: 366 / 7163 (5.110%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 565 mapped methods
- `mc-nbt`: 304 mapped methods
- `mc-protocol`: 273 mapped methods
- `mc-worldgen`: 334 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.chat.OutgoingChatMessage` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.chat.FontDescription` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.chat.ComponentContents` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.nbt.SnbtOperations` -> `mc-nbt` (376.1s)
- [PASS] `net.minecraft.nbt.visitors.CollectToTag` -> `mc-nbt` (257.9s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset` -> `mc-worldgen` (267.4s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLayerInfo` -> `mc-worldgen` (267.4s)
- [PASS] `net.minecraft.nbt.StringTag` -> `mc-nbt` (274.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChunkBatchStartPacket` -> `mc-protocol` (266.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundClientTickEndPacket` -> `mc-protocol` (266.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChangeGameModePacket` -> `mc-protocol` (266.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetHeldSlotPacket` -> `mc-protocol` (266.1s)
- [FAIL] `net.minecraft.core.Cursor3D` -> `mc-math` (202.2s)
- [FAIL] `net.minecraft.core.HolderGetter` -> `mc-math` (202.2s)
- [PASS] `net.minecraft.core.IdMapper` -> `mc-math` (202.2s)
