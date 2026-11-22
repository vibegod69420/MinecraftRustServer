# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:30:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1166 / 66676 (1.749%)
- **Mapped Java Classes**: 350 / 7163 (4.886%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 565 mapped methods
- `mc-nbt`: 267 mapped methods
- `mc-protocol`: 251 mapped methods
- `mc-worldgen`: 331 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.StringTag` -> `mc-nbt` (274.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChunkBatchStartPacket` -> `mc-protocol` (266.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundClientTickEndPacket` -> `mc-protocol` (266.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChangeGameModePacket` -> `mc-protocol` (266.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetHeldSlotPacket` -> `mc-protocol` (266.1s)
- [FAIL] `net.minecraft.core.Cursor3D` -> `mc-math` (202.2s)
- [FAIL] `net.minecraft.core.HolderGetter` -> `mc-math` (202.2s)
- [PASS] `net.minecraft.core.IdMapper` -> `mc-math` (202.2s)
- [PASS] `net.minecraft.core.Cloner` -> `mc-math` (202.2s)
- [PASS] `net.minecraft.network.protocol.common.custom.DiscardedPayload` -> `mc-protocol` (254.3s)
- [PASS] `net.minecraft.network.protocol.common.custom.BrandPayload` -> `mc-protocol` (254.3s)
- [PASS] `net.minecraft.nbt.IntArrayTag` -> `mc-nbt` (247.5s)
- [FAIL] `net.minecraft.core.component.TypedDataComponent` -> `mc-math` (549.9s)
- [FAIL] `net.minecraft.core.component.DataComponentGetter` -> `mc-math` (549.9s)
- [PASS] `net.minecraft.core.component.DataComponentHolder` -> `mc-math` (549.9s)
