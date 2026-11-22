# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:50:34 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1827 / 66676 (2.740%)
- **Mapped Java Classes**: 522 / 7163 (7.287%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 722 mapped methods
- `mc-nbt`: 518 mapped methods
- `mc-protocol`: 420 mapped methods
- `mc-worldgen`: 523 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket` -> `mc-protocol` (509.4s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundRecipeBookSeenRecipePacket` -> `mc-protocol` (509.4s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatCommandPacket` -> `mc-protocol` (509.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetTitleTextPacket` -> `mc-protocol` (509.4s)
- [FAIL] `net.minecraft.nbt.SnbtGrammar` -> `mc-nbt` (1149.4s)
- [PASS] `net.minecraft.nbt.SnbtPrinterTagVisitor` -> `mc-nbt` (1340.5s)
- [PASS] `net.minecraft.core.component.predicates.AttributeModifiersPredicate` -> `mc-math` (199.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChatAckPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugEventPacket` -> `mc-protocol` (564.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockChangedAckPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPickItemFromEntityPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetActionBarTextPacket` -> `mc-protocol` (218.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetSubtitleTextPacket` -> `mc-protocol` (218.0s)
