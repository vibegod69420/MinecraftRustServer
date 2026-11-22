# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:30:27 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1201 / 66676 (1.801%)
- **Mapped Java Classes**: 356 / 7163 (4.970%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 565 mapped methods
- `mc-nbt`: 299 mapped methods
- `mc-protocol`: 251 mapped methods
- `mc-worldgen`: 334 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPreset` -> `mc-worldgen` (267.4s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLayerInfo` -> `mc-worldgen` (267.4s)
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
