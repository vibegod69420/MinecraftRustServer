# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 20:50:24 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 135 / 66676 (0.203%)
- **Mapped Java Classes**: 28 / 7163 (0.391%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 129 mapped methods
- `mc-nbt`: 6 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.profiling.jfr.stats.TimedStat` -> `mc-math` (131.1s)
- [FAIL] `net.minecraft.util.parsing.packrat.NamedRule` -> `mc-math` (543.6s)
- [PASS] `net.minecraft.util.ToFloatFunction` -> `mc-math` (119.4s)
