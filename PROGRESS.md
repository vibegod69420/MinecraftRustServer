# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:08:16 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5388 / 66676 (8.081%)
- **Mapped Java Classes**: 1259 / 7163 (17.576%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1901 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 1995 mapped methods
- `mc-worldgen`: 1323 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (1026.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature` -> `mc-worldgen` (1026.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BonusChestFeature` -> `mc-worldgen` (1026.2s)
- [FAIL] `net.minecraft.network.HashedStack` -> `mc-protocol` (978.4s)
- [FAIL] `net.minecraft.network.PacketProcessor` -> `mc-protocol` (978.4s)
- [FAIL] `net.minecraft.network.PacketDecoder` -> `mc-protocol` (978.4s)
- [PASS] `net.minecraft.network.PacketEncoder` -> `mc-protocol` (978.4s)
- [FAIL] `net.minecraft.util.thread.AbstractConsecutiveExecutor` -> `mc-math` (961.0s)
- [PASS] `net.minecraft.util.worldupdate.WorldUpgrader` -> `mc-math` (310.4s)
- [FAIL] `net.minecraft.util.FastBufferedInputStream` -> `mc-math` (780.7s)
- [FAIL] `net.minecraft.util.SequencedPriorityIterator` -> `mc-math` (780.7s)
- [PASS] `net.minecraft.util.ByIdMap` -> `mc-math` (780.7s)
- [PASS] `net.minecraft.util.EncoderCache` -> `mc-math` (780.7s)
- [FAIL] `net.minecraft.util.CubicSpline` -> `mc-math` (250.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.DripstoneClusterConfiguration` -> `mc-worldgen` (172.7s)
