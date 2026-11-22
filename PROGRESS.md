# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:50:39 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2973 / 66676 (4.459%)
- **Mapped Java Classes**: 743 / 7163 (10.373%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1084 mapped methods
- `mc-nbt`: 1002 mapped methods
- `mc-protocol`: 783 mapped methods
- `mc-worldgen`: 704 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.util.MemoryReserve` -> `mc-math` (230.3s)
- [PASS] `net.minecraft.util.ExceptionCollector` -> `mc-math` (230.3s)
- [FAIL] `net.minecraft.core.component.PatchedDataComponentMap` -> `mc-math` (1153.4s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.BinomialDistributionGenerator` -> `mc-nbt` (405.1s)
