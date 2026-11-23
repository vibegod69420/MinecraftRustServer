# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:46:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8451 / 66676 (12.675%)
- **Mapped Java Classes**: 1879 / 7163 (26.232%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2890 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (1401.4s)
- [FAIL] `net.minecraft.util.debugchart.AbstractSampleLogger` -> `mc-math` (1363.7s)
- [FAIL] `net.minecraft.util.debugchart.LocalSampleLogger` -> `mc-math` (1363.7s)
- [PASS] `net.minecraft.util.debugchart.RemoteSampleLogger` -> `mc-math` (1363.7s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (1323.6s)
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
