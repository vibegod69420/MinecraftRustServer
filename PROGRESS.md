# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:40:03 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7200 / 66676 (10.799%)
- **Mapped Java Classes**: 1546 / 7163 (21.583%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1656 mapped methods
- `mc-protocol`: 3372 mapped methods
- `mc-worldgen`: 1920 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (778.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` -> `mc-worldgen` (769.2s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (596.9s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.TemplateStructurePiece` -> `mc-worldgen` (684.4s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket` -> `mc-protocol` (1036.6s)
