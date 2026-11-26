# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-26 00:50:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9761 / 66676 (14.639%)
- **Mapped Java Classes**: 2207 / 7163 (30.811%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1963 mapped methods
- `mc-protocol`: 3640 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.chunk.storage.SectionStorage` -> `mc-worldgen` (2607.6s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (2557.4s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (2515.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkProtoTickListFix` -> `mc-math` (827.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.GameRuleRegistryFix` -> `mc-math` (904.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.StatsCounterFix` -> `mc-math` (542.1s)
- [FAIL] `net.minecraft.util.profiling.metrics.profiling.ActiveMetricsRecorder` -> `mc-math` (1774.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.NewVillageFix` -> `mc-math` (1812.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix` -> `mc-math` (1812.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.MemoryExpiryDataFix` -> `mc-math` (1812.3s)
- [FAIL] `net.minecraft.util.parsing.packrat.commands.Grammar` -> `mc-math` (1720.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.AbstractBlockPropertyFix` -> `mc-math` (1784.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.BlockPropertyRenameAndFix` -> `mc-math` (1784.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkTicketUnpackPosFix` -> `mc-math` (1784.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityFallDistanceFloatToDoubleFix` -> `mc-math` (1784.8s)
