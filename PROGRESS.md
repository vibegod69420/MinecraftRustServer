# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:51:48 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3006 / 66676 (4.508%)
- **Mapped Java Classes**: 752 / 7163 (10.498%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1090 mapped methods
- `mc-nbt`: 1002 mapped methods
- `mc-protocol`: 783 mapped methods
- `mc-worldgen`: 740 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.MappedRegistry` -> `mc-math` (907.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer` -> `mc-worldgen` (683.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacerType` -> `mc-worldgen` (683.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer` -> `mc-worldgen` (683.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSelectTradePacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundRenameItemPacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.world.level.storage.loot.parameters.LootContextParams` -> `mc-nbt` (112.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket` -> `mc-protocol` (318.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundResetScorePacket` -> `mc-protocol` (318.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatSessionUpdatePacket` -> `mc-protocol` (318.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlaceGhostRecipePacket` -> `mc-protocol` (318.8s)
- [PASS] `net.minecraft.util.LenientJsonParser` -> `mc-math` (230.3s)
- [PASS] `net.minecraft.util.Keyframe` -> `mc-math` (230.3s)
