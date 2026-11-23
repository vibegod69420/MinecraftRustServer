# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:40:33 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8431 / 66676 (12.645%)
- **Mapped Java Classes**: 1873 / 7163 (26.148%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2861 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.schemas.V4302` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V3938` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V3325` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V2831` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityFieldsRenameFix` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsLowerCaseLanguageFix` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsRenameFieldFix` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.LodestoneCompassComponentFix` -> `mc-math` (994.8s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseChunk` -> `mc-worldgen` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.JigsawPropertiesFix` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityItemFrameDirectionFix` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkDeleteIgnoredLightDataFix` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkDeleteLightFix` -> `mc-math` (994.8s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (994.8s)
- [PASS] `net.minecraft.util.datafix.fixes.GoatHornIdFix` -> `mc-math` (994.7s)
