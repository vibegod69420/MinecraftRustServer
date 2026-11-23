# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 10:22:23 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8815 / 66676 (13.221%)
- **Mapped Java Classes**: 2003 / 7163 (27.963%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3423 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.LegacyDimensionIdFix` -> `mc-math` (1464.1s)
- [PASS] `net.minecraft.util.datafix.fixes.RaidRenamesDataFix` -> `mc-math` (1464.1s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkStatusFix2` -> `mc-math` (1464.1s)
- [PASS] `net.minecraft.util.datafix.fixes.FixWolfHealth` -> `mc-math` (1464.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (3495.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkBiomeFix` -> `mc-math` (3495.6s)
- [PASS] `net.minecraft.util.datafix.fixes.EmptyItemInHotbarFix` -> `mc-math` (3495.6s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackMapIdFix` -> `mc-math` (3495.6s)
- [FAIL] `net.minecraft.util.profiling.jfr.stats.GcHeapStat` -> `mc-math` (2480.3s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.IoSummary` -> `mc-math` (2480.3s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.ThreadAllocationStat` -> `mc-math` (2480.3s)
- [PASS] `net.minecraft.util.profiling.jfr.event.NetworkSummaryEvent` -> `mc-math` (1306.3s)
- [PASS] `net.minecraft.util.profiling.jfr.event.ChunkGenerationEvent` -> `mc-math` (1306.3s)
- [PASS] `net.minecraft.util.profiling.jfr.event.StructureGenerationEvent` -> `mc-math` (1306.3s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockNameFlatteningFix` -> `mc-math` (2189.2s)
