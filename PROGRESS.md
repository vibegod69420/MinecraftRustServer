# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 13:48:29 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9163 / 66676 (13.743%)
- **Mapped Java Classes**: 2104 / 7163 (29.373%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3901 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix` -> `mc-math` (4007.6s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsKeyLwjgl3Fix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.TooltipDisplayComponentFix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.TrappedChestBlockEntityFix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityEquipmentToArmorAndHandFix` -> `mc-math` (1610.9s)
- [FAIL] `net.minecraft.util.datafix.fixes.BitStorageAlignFix` -> `mc-math` (4007.5s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix` -> `mc-math` (1610.9s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackEnchantmentNamesFix` -> `mc-math` (4329.9s)
- [PASS] `net.minecraft.util.datafix.fixes.BannerPatternFormatFix` -> `mc-math` (4329.9s)
- [FAIL] `net.minecraft.util.datafix.DataFixTypes` -> `mc-math` (4329.9s)
- [PASS] `net.minecraft.util.datafix.fixes.RecipesFix` -> `mc-math` (4007.5s)
- [PASS] `net.minecraft.util.datafix.fixes.InlineBlockPosFormatFix` -> `mc-math` (764.3s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkBedBlockEntityInjecterFix` -> `mc-math` (2396.7s)
- [PASS] `net.minecraft.util.datafix.fixes.LegacyHoverEventFix` -> `mc-math` (2186.0s)
- [PASS] `net.minecraft.util.datafix.fixes.LevelFlatGeneratorInfoFix` -> `mc-math` (2186.0s)
