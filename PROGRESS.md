# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:12:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3327 / 66676 (4.990%)
- **Mapped Java Classes**: 828 / 7163 (11.559%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1208 mapped methods
- `mc-nbt`: 1106 mapped methods
- `mc-protocol`: 891 mapped methods
- `mc-worldgen`: 820 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.util.thread.PriorityConsecutiveExecutor` -> `mc-math` (1284.7s)
- [PASS] `net.minecraft.util.thread.TaskScheduler` -> `mc-math` (1284.7s)
- [PASS] `net.minecraft.util.thread.ReentrantBlockableEventLoop` -> `mc-math` (1284.7s)
- [PASS] `net.minecraft.util.thread.ConsecutiveExecutor` -> `mc-math` (1284.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.ThreeLayersFeatureSize` -> `mc-worldgen` (979.6s)
