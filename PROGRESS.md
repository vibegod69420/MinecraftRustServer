# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 06:04:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7499 / 66676 (11.247%)
- **Mapped Java Classes**: 1588 / 7163 (22.169%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2152 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalPiece` -> `mc-worldgen` (717.4s)
- [FAIL] `net.minecraft.world.level.levelgen.carver.CaveWorldCarver` -> `mc-worldgen` (1545.9s)
- [FAIL] `net.minecraft.network.chat.contents.TranslatableContents` -> `mc-protocol` (1515.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (781.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (1143.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (1396.3s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (609.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (1129.3s)
- [PASS] `net.minecraft.server.ServerTickRateManager` -> `mc-protocol` (1071.7s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (1046.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (331.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (839.4s)
- [PASS] `net.minecraft.world.level.levelgen.RandomState` -> `mc-worldgen` (822.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (723.8s)
- [PASS] `net.minecraft.world.level.levelgen.Noises` -> `mc-worldgen` (644.8s)
