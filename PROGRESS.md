# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:07:38 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8183 / 66676 (12.273%)
- **Mapped Java Classes**: 1777 / 7163 (24.808%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2512 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2636 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.profiling.jfr.event.PacketEvent` -> `mc-math` (162.2s)
- [PASS] `net.minecraft.util.profiling.jfr.event.ServerTickTimeEvent` -> `mc-math` (162.2s)
- [PASS] `net.minecraft.util.profiling.jfr.event.ClientFpsEvent` -> `mc-math` (162.2s)
- [PASS] `net.minecraft.util.profiling.jfr.event.ChunkRegionWriteEvent` -> `mc-math` (162.2s)
- [PASS] `net.minecraft.util.datafix.fixes.EntityWolfColorFix` -> `mc-math` (687.7s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityKeepPacked` -> `mc-math` (687.7s)
- [PASS] `net.minecraft.util.datafix.fixes.MapIdFix` -> `mc-math` (687.7s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemStackTagRemainderFix` -> `mc-math` (687.7s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunctions` -> `mc-worldgen` (1784.9s)
- [FAIL] `net.minecraft.world.level.levelgen.blending.Blender` -> `mc-worldgen` (1775.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V3082` -> `mc-math` (182.4s)
- [PASS] `net.minecraft.util.datafix.schemas.V2505` -> `mc-math` (182.4s)
- [PASS] `net.minecraft.util.datafix.schemas.V3825` -> `mc-math` (182.4s)
- [PASS] `net.minecraft.util.datafix.schemas.V3808` -> `mc-math` (182.4s)
- [PASS] `net.minecraft.util.datafix.fixes.WorldBorderWarningTimeFix` -> `mc-math` (258.1s)
