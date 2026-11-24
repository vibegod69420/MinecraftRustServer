# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 00:22:19 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9513 / 66676 (14.268%)
- **Mapped Java Classes**: 2162 / 7163 (30.183%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4143 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3478 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.datafix.PackedBitStorage` -> `mc-math` (4065.7s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStep` -> `mc-worldgen` (4926.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityPaintingMotiveFix` -> `mc-math` (4096.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHealthFix` -> `mc-math` (4096.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.AddFieldFix` -> `mc-math` (4096.0s)
- [PASS] `net.minecraft.util.datafix.fixes.ChunkBiomeFix` -> `mc-math` (4096.0s)
- [FAIL] `net.minecraft.world.level.levelgen.SurfaceRules` -> `mc-worldgen` (4989.3s)
- [FAIL] `net.minecraft.util.parsing.packrat.Dictionary` -> `mc-math` (3251.2s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (4829.6s)
- [FAIL] `net.minecraft.world.level.levelgen.blending.Blender` -> `mc-worldgen` (5278.7s)
- [PASS] `net.minecraft.util.datafix.schemas.V2832` -> `mc-math` (441.2s)
- [FAIL] `net.minecraft.world.level.chunk.status.ChunkStatusTasks` -> `mc-worldgen` (2028.6s)
- [FAIL] `net.minecraft.world.level.levelgen.NoiseRouterData` -> `mc-worldgen` (1927.2s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (1670.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (2296.8s)
