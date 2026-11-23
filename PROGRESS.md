# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:51:44 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6092 / 66676 (9.137%)
- **Mapped Java Classes**: 1361 / 7163 (19.000%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2458 mapped methods
- `mc-worldgen`: 1584 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket` -> `mc-protocol` (498.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMerchantOffersPacket` -> `mc-protocol` (498.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSoundEntityPacket` -> `mc-protocol` (498.4s)
- [FAIL] `net.minecraft.network.HashedPatchMap` -> `mc-protocol` (989.1s)
- [PASS] `net.minecraft.network.LpVec3` -> `mc-protocol` (989.1s)
- [PASS] `net.minecraft.network.CompressionDecoder` -> `mc-protocol` (989.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.RootSystemFeature` -> `mc-worldgen` (415.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.MineshaftStructure` -> `mc-worldgen` (247.1s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket` -> `mc-protocol` (653.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundLoginPacket` -> `mc-protocol` (653.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket` -> `mc-protocol` (653.8s)
- [PASS] `net.minecraft.world.level.levelgen.PatrolSpawner` -> `mc-worldgen` (256.1s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket` -> `mc-protocol` (774.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundContainerClickPacket` -> `mc-protocol` (774.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundStopSoundPacket` -> `mc-protocol` (774.8s)
