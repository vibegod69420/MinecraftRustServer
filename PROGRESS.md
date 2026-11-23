# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:26:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7175 / 66676 (10.761%)
- **Mapped Java Classes**: 1542 / 7163 (21.527%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 1892 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (778.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` -> `mc-worldgen` (769.2s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (596.9s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.TemplateStructurePiece` -> `mc-worldgen` (684.4s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket` -> `mc-protocol` (1036.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBossEventPacket` -> `mc-protocol` (637.6s)
- [FAIL] `net.minecraft.network.chat.contents.TranslatableContents` -> `mc-protocol` (816.6s)
- [PASS] `net.minecraft.server.TickTask` -> `mc-protocol` (103.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket` -> `mc-protocol` (965.6s)
- [PASS] `net.minecraft.network.protocol.game.GameProtocols` -> `mc-protocol` (217.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` -> `mc-protocol` (892.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket` -> `mc-protocol` (892.2s)
- [FAIL] `net.minecraft.network.chat.ComponentUtils` -> `mc-protocol` (876.7s)
- [PASS] `net.minecraft.network.chat.SignedMessageChain` -> `mc-protocol` (812.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundMovePlayerPacket` -> `mc-protocol` (753.6s)
