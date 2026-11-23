# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:42:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6452 / 66676 (9.677%)
- **Mapped Java Classes**: 1415 / 7163 (19.754%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2824 mapped methods
- `mc-worldgen`: 1688 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.chat.LastSeenMessagesValidator` -> `mc-protocol` (375.0s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (984.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDamageEventPacket` -> `mc-protocol` (915.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerLookAtPacket` -> `mc-protocol` (915.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket` -> `mc-protocol` (915.2s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (832.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BasaltPillarFeature` -> `mc-worldgen` (322.2s)
- [FAIL] `net.minecraft.network.HashedPatchMap` -> `mc-protocol` (710.7s)
- [FAIL] `net.minecraft.network.PacketProcessor` -> `mc-protocol` (710.7s)
- [FAIL] `net.minecraft.network.PacketDecoder` -> `mc-protocol` (710.7s)
- [FAIL] `net.minecraft.network.chat.contents.data.BlockDataSource` -> `mc-protocol` (674.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (657.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator` -> `mc-worldgen` (657.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.DarkOakTrunkPlacer` -> `mc-worldgen` (650.9s)
- [FAIL] `net.minecraft.network.chat.SignableCommand` -> `mc-protocol` (635.4s)
