# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 21:48:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 186 / 66676 (0.279%)
- **Mapped Java Classes**: 50 / 7163 (0.698%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 229 mapped methods
- `mc-nbt`: 6 mapped methods
- `mc-worldgen`: 4 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (103.0s)
- [PASS] `net.minecraft.util.random.Weighted` -> `mc-math` (43.4s)
- [PASS] `net.minecraft.util.profiling.metrics.profiling.MetricsRecorder` -> `mc-math` (124.0s)
- [PASS] `net.minecraft.util.RegistryContextSwapper` -> `mc-math` (162.7s)
- [PASS] `net.minecraft.util.profiling.metrics.MetricsSamplerProvider` -> `mc-math` (201.9s)
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
