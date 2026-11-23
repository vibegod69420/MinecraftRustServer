# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 10:42:21 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8852 / 66676 (13.276%)
- **Mapped Java Classes**: 2012 / 7163 (28.089%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3462 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.FeatureFlagRemoveFix` -> `mc-math` (2285.0s)
- [PASS] `net.minecraft.util.datafix.fixes.AbstractArrowPickupFix` -> `mc-math` (2285.0s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityZombieSplitFix` -> `mc-math` (2285.0s)
- [PASS] `net.minecraft.util.datafix.fixes.ReorganizePoi` -> `mc-math` (2285.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHorseSaddleFix` -> `mc-math` (2198.6s)
- [PASS] `net.minecraft.util.datafix.fixes.AbstractPoiSectionFix` -> `mc-math` (2198.6s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackUUIDFix` -> `mc-math` (2198.6s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackTagFix` -> `mc-math` (2198.6s)
- [PASS] `net.minecraft.util.datafix.fixes.LegacyDimensionIdFix` -> `mc-math` (1464.1s)
- [PASS] `net.minecraft.util.datafix.fixes.RaidRenamesDataFix` -> `mc-math` (1464.1s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkStatusFix2` -> `mc-math` (1464.1s)
- [PASS] `net.minecraft.util.datafix.fixes.FixWolfHealth` -> `mc-math` (1464.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (3495.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkBiomeFix` -> `mc-math` (3495.6s)
- [PASS] `net.minecraft.util.datafix.fixes.EmptyItemInHotbarFix` -> `mc-math` (3495.6s)
