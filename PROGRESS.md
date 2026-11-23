# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 04:36:03 UTC
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
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.JigsawStructure` -> `mc-worldgen` (304.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.VegetationPatchFeature` -> `mc-worldgen` (1456.1s)
- [PASS] `net.minecraft.world.level.levelgen.NoiseGeneratorSettings` -> `mc-worldgen` (243.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.BuiltInLootTables` -> `mc-nbt` (624.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.trunkplacers.UpwardsBranchingTrunkPlacer` -> `mc-worldgen` (778.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` -> `mc-worldgen` (769.2s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (596.9s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.TemplateStructurePiece` -> `mc-worldgen` (684.4s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket` -> `mc-protocol` (1036.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBossEventPacket` -> `mc-protocol` (637.6s)
- [FAIL] `net.minecraft.network.chat.contents.TranslatableContents` -> `mc-protocol` (816.6s)
- [PASS] `net.minecraft.server.TickTask` -> `mc-protocol` (103.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPlayerTeamPacket` -> `mc-protocol` (965.6s)
- [PASS] `net.minecraft.network.protocol.game.GameProtocols` -> `mc-protocol` (217.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` -> `mc-protocol` (892.2s)
