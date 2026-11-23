# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 08:08:25 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8192 / 66676 (12.286%)
- **Mapped Java Classes**: 1781 / 7163 (24.864%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2529 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2636 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.datafix.schemas.V3818` -> `mc-math` (208.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V1928` -> `mc-math` (208.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V3204` -> `mc-math` (208.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V1451` -> `mc-math` (208.8s)
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
