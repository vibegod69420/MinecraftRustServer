# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 21:04:04 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 150 / 66676 (0.225%)
- **Mapped Java Classes**: 34 / 7163 (0.475%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 154 mapped methods
- `mc-nbt`: 6 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.valueproviders.SampledFloat` -> `mc-math` (137.4s)
- [PASS] `net.minecraft.util.profiling.metrics.ProfilerMeasured` -> `mc-math` (110.0s)
- [PASS] `net.minecraft.util.CryptException` -> `mc-math` (119.5s)
- [PASS] `net.minecraft.util.profiling.jfr.callback.ProfiledDuration` -> `mc-math` (119.7s)
- [PASS] `net.minecraft.util.debugchart.TpsDebugDimensions` -> `mc-math` (133.4s)
- [PASS] `net.minecraft.util.parsing.packrat.ErrorEntry` -> `mc-math` (199.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TimedStat` -> `mc-math` (131.1s)
- [FAIL] `net.minecraft.util.parsing.packrat.NamedRule` -> `mc-math` (543.6s)
- [PASS] `net.minecraft.util.ToFloatFunction` -> `mc-math` (119.4s)
