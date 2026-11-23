# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:14:16 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7015 / 66676 (10.521%)
- **Mapped Java Classes**: 1513 / 7163 (21.122%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3225 mapped methods
- `mc-worldgen`: 1866 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.NetherFossilPieces` -> `mc-worldgen` (296.8s)
- [PASS] `net.minecraft.network.chat.ChatType` -> `mc-protocol` (392.3s)
- [PASS] `net.minecraft.network.chat.contents.ScoreContents` -> `mc-protocol` (373.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundInteractPacket` -> `mc-protocol` (373.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMoveEntityPacket` -> `mc-protocol` (373.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerInfoUpdatePacket` -> `mc-protocol` (277.3s)
- [PASS] `net.minecraft.network.chat.ClickEvent` -> `mc-protocol` (172.3s)
- [PASS] `net.minecraft.network.protocol.configuration.ConfigurationProtocols` -> `mc-protocol` (147.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (1068.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement` -> `mc-worldgen` (264.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (670.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundAddEntityPacket` -> `mc-protocol` (342.8s)
- [PASS] `net.minecraft.world.level.storage.TagValueInput` -> `mc-nbt` (855.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece` -> `mc-worldgen` (167.2s)
- [PASS] `net.minecraft.network.chat.ChatTypeDecoration` -> `mc-protocol` (165.3s)
