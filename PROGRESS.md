# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:49:09 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1782 / 66676 (2.673%)
- **Mapped Java Classes**: 513 / 7163 (7.162%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 722 mapped methods
- `mc-nbt`: 496 mapped methods
- `mc-protocol`: 400 mapped methods
- `mc-worldgen`: 516 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatAckPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugEventPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetActionBarTextPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.HasSturdyFacePredicate` -> `mc-worldgen` (201.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.WouldSurvivePredicate` -> `mc-worldgen` (201.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.MatchingFluidsPredicate` -> `mc-worldgen` (201.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.MatchingBlocksPredicate` -> `mc-worldgen` (201.3s)
- [FAIL] `net.minecraft.core.DefaultedMappedRegistry` -> `mc-math` (584.7s)
- [PASS] `net.minecraft.core.AxisCycle` -> `mc-math` (584.7s)
- [PASS] `net.minecraft.core.FrontAndTop` -> `mc-math` (584.7s)
