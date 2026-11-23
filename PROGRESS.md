# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:22:50 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7129 / 66676 (10.692%)
- **Mapped Java Classes**: 1533 / 7163 (21.402%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3326 mapped methods
- `mc-worldgen`: 1892 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket` -> `mc-protocol` (965.6s)
- [PASS] `net.minecraft.network.protocol.game.GameProtocols` -> `mc-protocol` (217.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` -> `mc-protocol` (892.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket` -> `mc-protocol` (892.2s)
- [FAIL] `net.minecraft.network.chat.ComponentUtils` -> `mc-protocol` (876.7s)
- [PASS] `net.minecraft.network.chat.SignedMessageChain` -> `mc-protocol` (812.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundMovePlayerPacket` -> `mc-protocol` (753.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.FallenTreeFeature` -> `mc-worldgen` (704.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool` -> `mc-worldgen` (198.2s)
- [PASS] `net.minecraft.network.syncher.EntityDataSerializers` -> `mc-protocol` (183.6s)
- [PASS] `net.minecraft.network.chat.contents.NbtContents` -> `mc-protocol` (487.1s)
- [PASS] `net.minecraft.network.chat.HoverEvent` -> `mc-protocol` (469.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces` -> `mc-worldgen` (296.8s)
- [PASS] `net.minecraft.network.chat.ChatType` -> `mc-protocol` (392.3s)
- [PASS] `net.minecraft.network.chat.contents.ScoreContents` -> `mc-protocol` (373.1s)
