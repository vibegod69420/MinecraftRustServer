# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:18:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7059 / 66676 (10.587%)
- **Mapped Java Classes**: 1519 / 7163 (21.206%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3247 mapped methods
- `mc-worldgen`: 1892 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.FallenTreeFeature` -> `mc-worldgen` (704.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.StructureTemplatePool` -> `mc-worldgen` (198.2s)
- [PASS] `net.minecraft.network.syncher.EntityDataSerializers` -> `mc-protocol` (183.6s)
- [PASS] `net.minecraft.network.chat.contents.NbtContents` -> `mc-protocol` (487.1s)
- [PASS] `net.minecraft.network.chat.HoverEvent` -> `mc-protocol` (469.0s)
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
