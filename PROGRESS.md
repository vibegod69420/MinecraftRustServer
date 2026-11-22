# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 21:20:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 163 / 66676 (0.244%)
- **Mapped Java Classes**: 39 / 7163 (0.544%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 183 mapped methods
- `mc-nbt`: 6 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.profiling.ProfilerPathEntry` -> `mc-math` (328.2s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.FpsStat` -> `mc-math` (127.9s)
- [PASS] `net.minecraft.util.debugchart.SampleLogger` -> `mc-math` (82.5s)
- [PASS] `net.minecraft.util.debugchart.SampleStorage` -> `mc-math` (158.3s)
- [PASS] `net.minecraft.util.FormattedCharSink` -> `mc-math` (204.5s)
- [FAIL] `net.minecraft.util.valueproviders.SampledFloat` -> `mc-math` (137.4s)
- [PASS] `net.minecraft.util.profiling.metrics.ProfilerMeasured` -> `mc-math` (110.0s)
- [PASS] `net.minecraft.util.CryptException` -> `mc-math` (119.5s)
- [PASS] `net.minecraft.util.profiling.jfr.callback.ProfiledDuration` -> `mc-math` (119.7s)
- [PASS] `net.minecraft.util.debugchart.TpsDebugDimensions` -> `mc-math` (133.4s)
- [PASS] `net.minecraft.util.parsing.packrat.ErrorEntry` -> `mc-math` (199.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TimedStat` -> `mc-math` (131.1s)
- [FAIL] `net.minecraft.util.parsing.packrat.NamedRule` -> `mc-math` (543.6s)
- [PASS] `net.minecraft.util.ToFloatFunction` -> `mc-math` (119.4s)
