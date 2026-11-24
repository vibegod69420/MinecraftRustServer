# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 15:22:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9652 / 66676 (14.476%)
- **Mapped Java Classes**: 2175 / 7163 (30.364%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1864 mapped methods
- `mc-protocol`: 3587 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (1314.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.RuinedPortalStructure` -> `mc-worldgen` (894.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneClusterFeature` -> `mc-worldgen` (1207.0s)
- [FAIL] `net.minecraft.world.level.levelgen.WorldDimensions` -> `mc-worldgen` (1069.7s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (1752.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (795.4s)
- [PASS] `net.minecraft.network.syncher.SynchedEntityData` -> `mc-protocol` (1703.4s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (1599.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootTable` -> `mc-nbt` (1582.4s)
- [PASS] `net.minecraft.network.Connection` -> `mc-protocol` (1555.9s)
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (746.0s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (1485.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (1379.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (1208.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (1178.8s)
