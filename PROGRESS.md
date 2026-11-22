# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 21:34:53 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 170 / 66676 (0.255%)
- **Mapped Java Classes**: 42 / 7163 (0.586%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 196 mapped methods
- `mc-nbt`: 6 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.parsing.packrat.SuggestionSupplier` -> `mc-math` (593.6s)
- [PASS] `net.minecraft.util.KeyDispatchDataCodec` -> `mc-math` (209.2s)
- [PASS] `net.minecraft.util.parsing.packrat.Atom` -> `mc-math` (143.8s)
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
