# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 20:54:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9407 / 66676 (14.108%)
- **Mapped Java Classes**: 2141 / 7163 (29.890%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4092 mapped methods
- `mc-nbt`: 1769 mapped methods
- `mc-protocol`: 3450 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (62.1s)
- [PASS] `net.minecraft.world.level.storage.ValueInputContextHelper` -> `mc-nbt` (2759.8s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (11233.4s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction` -> `mc-nbt` (2531.4s)
- [FAIL] `net.minecraft.network.chat.SignableCommand` -> `mc-protocol` (12066.2s)
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
