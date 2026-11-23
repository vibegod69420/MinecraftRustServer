# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:00:56 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4398 / 66676 (6.596%)
- **Mapped Java Classes**: 1076 / 7163 (15.022%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1553 mapped methods
- `mc-nbt`: 1236 mapped methods
- `mc-protocol`: 1483 mapped methods
- `mc-worldgen`: 1202 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.PiecesContainer` -> `mc-worldgen` (554.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.PieceGeneratorSupplier` -> `mc-worldgen` (554.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.FoliagePlacerType` -> `mc-worldgen` (1118.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.MegaPineFoliagePlacer` -> `mc-worldgen` (1118.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.BlobFoliagePlacer` -> `mc-worldgen` (1118.1s)
- [FAIL] `net.minecraft.util.FastBufferedInputStream` -> `mc-math` (1342.1s)
- [FAIL] `net.minecraft.util.SequencedPriorityIterator` -> `mc-math` (1342.1s)
- [PASS] `net.minecraft.util.ClassInstanceMultiMap` -> `mc-math` (1342.1s)
- [PASS] `net.minecraft.util.StaticCache2D` -> `mc-math` (1342.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetLoreFunction` -> `mc-nbt` (358.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetContainerContents` -> `mc-nbt` (364.5s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundResourcePackPushPacket` -> `mc-protocol` (869.2s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket` -> `mc-protocol` (869.2s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundResourcePackPacket` -> `mc-protocol` (869.2s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket` -> `mc-protocol` (869.2s)
