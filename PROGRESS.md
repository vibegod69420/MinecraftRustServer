# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:58:12 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4359 / 66676 (6.538%)
- **Mapped Java Classes**: 1066 / 7163 (14.882%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1533 mapped methods
- `mc-nbt`: 1236 mapped methods
- `mc-protocol`: 1483 mapped methods
- `mc-worldgen`: 1170 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetEnchantmentsFunction` -> `mc-nbt` (275.3s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.NestedLootTable` -> `mc-nbt` (243.0s)
- [PASS] `net.minecraft.world.level.storage.CommandStorage` -> `mc-nbt` (174.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.CoralClawFeature` -> `mc-worldgen` (984.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockColumnFeature` -> `mc-worldgen` (984.6s)
