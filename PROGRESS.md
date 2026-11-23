# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:43:35 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8436 / 66676 (12.652%)
- **Mapped Java Classes**: 1875 / 7163 (26.176%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2866 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.fixes.PlayerUUIDFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.PlayerRespawnDataFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.EmptyItemInVillagerTradeFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.AdvancementsRenameFix` -> `mc-math` (1243.0s)
- [FAIL] `net.minecraft.world.level.chunk.UpgradeData` -> `mc-worldgen` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.PrimedTntBlockStateFixer` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.GossipUUIDFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityShulkerRotationFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.ContainerBlockEntityLockPredicateFix` -> `mc-math` (1243.0s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsMenuBlurrinessFix` -> `mc-math` (1242.9s)
- [PASS] `net.minecraft.util.datafix.fixes.StructureReferenceCountFix` -> `mc-math` (1242.9s)
- [PASS] `net.minecraft.util.datafix.fixes.CustomModelDataExpandFix` -> `mc-math` (1242.9s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsFancyGraphicsToGraphicsModeFix` -> `mc-math` (1242.9s)
- [PASS] `net.minecraft.util.datafix.schemas.V4302` -> `mc-math` (994.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V3938` -> `mc-math` (994.8s)
