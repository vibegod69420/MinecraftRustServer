# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 18:40:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9377 / 66676 (14.063%)
- **Mapped Java Classes**: 2139 / 7163 (29.862%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4090 mapped methods
- `mc-nbt`: 1736 mapped methods
- `mc-protocol`: 3450 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.DefaultedMappedRegistry` -> `mc-math` (9776.8s)
- [PASS] `net.minecraft.network.HashedPatchMap` -> `mc-protocol` (9546.7s)
- [PASS] `net.minecraft.network.PacketProcessor` -> `mc-protocol` (9546.7s)
- [PASS] `net.minecraft.network.PacketDecoder` -> `mc-protocol` (9546.7s)
- [PASS] `net.minecraft.util.GsonHelper` -> `mc-math` (1453.9s)
- [PASS] `net.minecraft.util.thread.AbstractConsecutiveExecutor` -> `mc-math` (9465.6s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (9535.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (9533.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (9533.7s)
- [FAIL] `net.minecraft.nbt.TextComponentTagVisitor` -> `mc-nbt` (9546.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature` -> `mc-worldgen` (9448.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (9448.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DeltaFeature` -> `mc-worldgen` (9448.5s)
- [FAIL] `net.minecraft.network.chat.contents.data.BlockDataSource` -> `mc-protocol` (9534.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (8535.4s)
