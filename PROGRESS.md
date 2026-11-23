# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 09:38:20 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8673 / 66676 (13.008%)
- **Mapped Java Classes**: 1954 / 7163 (27.279%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3212 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.schemas.V4292` -> `mc-math` (689.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V4312` -> `mc-math` (689.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V1510` -> `mc-math` (689.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V2842` -> `mc-math` (689.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityFallDistanceFloatToDoubleFix` -> `mc-math` (607.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.BlockPropertyRenameAndFix` -> `mc-math` (607.6s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityBannerColorFix` -> `mc-math` (607.6s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityMinecartIdentifiersFix` -> `mc-math` (607.6s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (607.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.MapBannerBlockPosFormatFix` -> `mc-math` (607.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityAttributeBaseValueFix` -> `mc-math` (607.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.WorldGenSettingsDisallowOldCustomWorldsFix` -> `mc-math` (607.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.MemoryExpiryDataFix` -> `mc-math` (607.6s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionPieces` -> `mc-worldgen` (607.6s)
- [PASS] `net.minecraft.util.datafix.schemas.V2501` -> `mc-math` (250.1s)
