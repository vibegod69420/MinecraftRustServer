# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:29:46 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6244 / 66676 (9.365%)
- **Mapped Java Classes**: 1384 / 7163 (19.322%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2632 mapped methods
- `mc-worldgen`: 1612 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerChatPacket` -> `mc-protocol` (388.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket` -> `mc-protocol` (388.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket` -> `mc-protocol` (388.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.VegetationPatchFeature` -> `mc-worldgen` (2129.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (1180.9s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CanyonWorldCarver` -> `mc-worldgen` (1352.5s)
- [PASS] `net.minecraft.network.chat.FormattedText` -> `mc-protocol` (720.0s)
- [PASS] `net.minecraft.network.chat.LastSeenMessages` -> `mc-protocol` (720.0s)
- [PASS] `net.minecraft.network.chat.SignedMessageValidator` -> `mc-protocol` (720.0s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket` -> `mc-protocol` (723.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSoundPacket` -> `mc-protocol` (723.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket` -> `mc-protocol` (723.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (832.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (692.2s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundPlayerChatPacket` -> `mc-protocol` (939.1s)
