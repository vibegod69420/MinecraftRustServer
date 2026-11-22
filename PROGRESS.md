# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 20:53:44 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 142 / 66676 (0.213%)
- **Mapped Java Classes**: 29 / 7163 (0.405%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 136 mapped methods
- `mc-nbt`: 6 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.parsing.packrat.ErrorEntry` -> `mc-math` (199.4s)
- [PASS] `net.minecraft.util.profiling.jfr.stats.TimedStat` -> `mc-math` (131.1s)
- [FAIL] `net.minecraft.util.parsing.packrat.NamedRule` -> `mc-math` (543.6s)
- [PASS] `net.minecraft.util.ToFloatFunction` -> `mc-math` (119.4s)
