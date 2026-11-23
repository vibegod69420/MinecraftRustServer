# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:58:04 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5271 / 66676 (7.905%)
- **Mapped Java Classes**: 1234 / 7163 (17.227%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1868 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 1926 mapped methods
- `mc-worldgen`: 1283 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPassengersPacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetExperiencePacket` -> `mc-protocol` (325.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundUseItemOnPacket` -> `mc-protocol` (325.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderLerpSizePacket` -> `mc-protocol` (325.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMountScreenOpenPacket` -> `mc-protocol` (325.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundMoveVehiclePacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatPacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket` -> `mc-protocol` (248.1s)
- [PASS] `net.minecraft.network.protocol.status.StatusProtocols` -> `mc-protocol` (162.1s)
- [PASS] `net.minecraft.network.chat.numbers.StyledFormat` -> `mc-protocol` (148.3s)
- [FAIL] `net.minecraft.world.level.storage.DimensionDataStorage` -> `mc-nbt` (2089.5s)
