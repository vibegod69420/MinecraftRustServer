# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:06:04 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4522 / 66676 (6.782%)
- **Mapped Java Classes**: 1097 / 7163 (15.315%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1577 mapped methods
- `mc-nbt`: 1298 mapped methods
- `mc-protocol`: 1518 mapped methods
- `mc-worldgen`: 1233 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundEntityEventPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction` -> `mc-nbt` (177.4s)
- [FAIL] `net.minecraft.world.level.storage.PlayerDataStorage` -> `mc-nbt` (1294.9s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket` -> `mc-protocol` (773.1s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket` -> `mc-protocol` (773.1s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundChatPacket` -> `mc-protocol` (773.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundDebugSubscriptionRequestPacket` -> `mc-protocol` (773.1s)
- [PASS] `net.minecraft.util.StringRepresentable` -> `mc-math` (201.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundProjectilePowerPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRemoveEntitiesPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundEntityTagQueryPacket` -> `mc-protocol` (883.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket` -> `mc-protocol` (883.1s)
