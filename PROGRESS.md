# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:50:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7231 / 66676 (10.845%)
- **Mapped Java Classes**: 1555 / 7163 (21.709%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 1952 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.FancyTrunkPlacer` -> `mc-worldgen` (1540.6s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (2020.7s)
- [PASS] `net.minecraft.world.level.levelgen.flat.FlatLevelGeneratorPresets` -> `mc-worldgen` (853.1s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.ShipwreckPieces` -> `mc-worldgen` (180.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (992.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.IglooPieces` -> `mc-worldgen` (1114.8s)
- [FAIL] `net.minecraft.server.ServerTickRateManager` -> `mc-protocol` (1061.4s)
- [FAIL] `net.minecraft.world.level.levelgen.carver.CaveWorldCarver` -> `mc-worldgen` (1332.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (871.5s)
- [FAIL] `net.minecraft.world.level.levelgen.Noises` -> `mc-worldgen` (708.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeFungusFeature` -> `mc-worldgen` (805.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.JigsawStructure` -> `mc-worldgen` (304.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.VegetationPatchFeature` -> `mc-worldgen` (1456.1s)
- [PASS] `net.minecraft.world.level.levelgen.NoiseGeneratorSettings` -> `mc-worldgen` (243.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.BuiltInLootTables` -> `mc-nbt` (624.2s)
