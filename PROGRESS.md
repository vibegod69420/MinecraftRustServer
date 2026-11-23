# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:13:45 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3381 / 66676 (5.071%)
- **Mapped Java Classes**: 843 / 7163 (11.769%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1217 mapped methods
- `mc-nbt`: 1113 mapped methods
- `mc-protocol`: 952 mapped methods
- `mc-worldgen`: 820 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.FileSystemUtil` -> `mc-math` (265.2s)
- [PASS] `net.minecraft.util.TaskChainer` -> `mc-math` (265.2s)
- [PASS] `net.minecraft.util.SpecialDates` -> `mc-math` (265.2s)
- [PASS] `net.minecraft.util.CommonColors` -> `mc-math` (265.2s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.ContextNbtProvider` -> `mc-nbt` (253.4s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.BlockPredicateType` -> `mc-worldgen` (242.2s)
- [PASS] `net.minecraft.util.BinaryAnimator` -> `mc-math` (230.0s)
- [PASS] `net.minecraft.util.Signer` -> `mc-math` (230.0s)
- [PASS] `net.minecraft.util.Graph` -> `mc-math` (230.0s)
- [PASS] `net.minecraft.util.SmoothDouble` -> `mc-math` (230.0s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.ScoreboardValue` -> `mc-nbt` (193.7s)
- [PASS] `net.minecraft.util.AbortableIterationConsumer` -> `mc-math` (1057.6s)
- [PASS] `net.minecraft.util.TimeUtil` -> `mc-math` (1057.6s)
- [PASS] `net.minecraft.util.SingleKeyCache` -> `mc-math` (1057.6s)
- [PASS] `net.minecraft.util.TickThrottler` -> `mc-math` (1057.6s)
