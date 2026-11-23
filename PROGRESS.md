# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 16:09:27 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9221 / 66676 (13.830%)
- **Mapped Java Classes**: 2114 / 7163 (29.513%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3976 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (604.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.CoralTreeFeature` -> `mc-worldgen` (604.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SeaPickleFeature` -> `mc-worldgen` (604.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SimpleBlockFeature` -> `mc-worldgen` (604.6s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (429.4s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (429.4s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (429.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (97.0s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (75.3s)
- [FAIL] `net.minecraft.util.datafix.fixes.AttributeModifierIdFix` -> `mc-math` (5848.2s)
- [FAIL] `net.minecraft.util.parsing.packrat.Term` -> `mc-math` (5544.8s)
- [FAIL] `net.minecraft.util.profiling.jfr.parse.JfrStatsParser` -> `mc-math` (5544.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.ParticleUnflatteningFix` -> `mc-math` (5544.8s)
- [FAIL] `net.minecraft.util.eventlog.EventLogDirectory` -> `mc-math` (5544.7s)
- [FAIL] `net.minecraft.util.profiling.FilledProfileResults` -> `mc-math` (5544.2s)
