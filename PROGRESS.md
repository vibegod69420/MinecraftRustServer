# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:11:22 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3303 / 66676 (4.954%)
- **Mapped Java Classes**: 819 / 7163 (11.434%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1182 mapped methods
- `mc-nbt`: 1106 mapped methods
- `mc-protocol`: 891 mapped methods
- `mc-worldgen`: 818 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.network.protocol.game.ClientboundDebugSamplePacket` -> `mc-protocol` (1217.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlaceRecipePacket` -> `mc-protocol` (1217.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundServerDataPacket` -> `mc-protocol` (1217.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerInfoRemovePacket` -> `mc-protocol` (1217.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.BonusLevelTableCondition` -> `mc-nbt` (1099.4s)
