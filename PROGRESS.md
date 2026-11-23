# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:06:53 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4547 / 66676 (6.819%)
- **Mapped Java Classes**: 1100 / 7163 (15.357%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1597 mapped methods
- `mc-nbt`: 1298 mapped methods
- `mc-protocol`: 1518 mapped methods
- `mc-worldgen`: 1241 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap` -> `mc-math` (230.0s)
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
