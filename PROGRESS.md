# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:00:19 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8157 / 66676 (12.234%)
- **Mapped Java Classes**: 1763 / 7163 (24.613%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2448 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2636 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.WorldBorderWarningTimeFix` -> `mc-math` (258.1s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityPufferfishRenameFix` -> `mc-math` (258.1s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityShulkerColorFix` -> `mc-math` (258.1s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityRavagerRenameFix` -> `mc-math` (258.1s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityArmorStandSilentFix` -> `mc-math` (287.1s)
- [PASS] `net.minecraft.util.datafix.fixes.RandomSequenceSettingsFix` -> `mc-math` (287.1s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsSetGraphicsPresetToCustomFix` -> `mc-math` (287.1s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsAccessibilityOnboardFix` -> `mc-math` (287.1s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (1736.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V2511_1` -> `mc-math` (405.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V3683` -> `mc-math` (405.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V1451_4` -> `mc-math` (405.3s)
- [PASS] `net.minecraft.util.datafix.schemas.V3438` -> `mc-math` (405.3s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsForceVBOFix` -> `mc-math` (589.7s)
- [PASS] `net.minecraft.util.datafix.fixes.CatTypeFix` -> `mc-math` (589.7s)
