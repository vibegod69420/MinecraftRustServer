# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 05:58:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 7484 / 66676 (11.224%)
- **Mapped Java Classes**: 1587 / 7163 (22.155%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2067 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2136 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.presets.WorldPresets` -> `mc-worldgen` (609.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.GeodeFeature` -> `mc-worldgen` (1129.3s)
- [PASS] `net.minecraft.server.ServerTickRateManager` -> `mc-protocol` (1071.7s)
- [FAIL] `net.minecraft.network.Connection` -> `mc-protocol` (1046.2s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheck` -> `mc-worldgen` (331.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.SwampHutPiece` -> `mc-worldgen` (839.4s)
- [PASS] `net.minecraft.world.level.levelgen.RandomState` -> `mc-worldgen` (822.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.LakeFeature` -> `mc-worldgen` (723.8s)
- [PASS] `net.minecraft.world.level.levelgen.Noises` -> `mc-worldgen` (644.8s)
- [FAIL] `net.minecraft.network.chat.ComponentUtils` -> `mc-protocol` (615.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetStructureBlockPacket` -> `mc-protocol` (610.7s)
- [PASS] `net.minecraft.world.level.levelgen.structure.TemplateStructurePiece` -> `mc-worldgen` (578.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.MonsterRoomFeature` -> `mc-worldgen` (280.2s)
- [PASS] `net.minecraft.world.level.chunk.storage.RegionBitmap` -> `mc-worldgen` (259.4s)
- [PASS] `net.minecraft.world.level.chunk.storage.LegacyTagFixer` -> `mc-worldgen` (259.4s)
