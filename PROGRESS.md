# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:59:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6131 / 66676 (9.195%)
- **Mapped Java Classes**: 1365 / 7163 (19.056%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2495 mapped methods
- `mc-worldgen`: 1603 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket` -> `mc-protocol` (723.8s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundSoundPacket` -> `mc-protocol` (723.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetCommandBlockPacket` -> `mc-protocol` (723.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DesertWellFeature` -> `mc-worldgen` (832.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer` -> `mc-worldgen` (692.2s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundPlayerChatPacket` -> `mc-protocol` (939.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerActionPacket` -> `mc-protocol` (939.1s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSeenAdvancementsPacket` -> `mc-protocol` (939.1s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.ListPoolElement` -> `mc-worldgen` (913.8s)
- [FAIL] `net.minecraft.world.level.levelgen.DensityFunction` -> `mc-worldgen` (1082.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DripstoneUtils` -> `mc-worldgen` (665.8s)
- [FAIL] `net.minecraft.network.chat.LastSeenMessagesTracker` -> `mc-protocol` (624.6s)
- [PASS] `net.minecraft.network.chat.SignedMessageBody` -> `mc-protocol` (624.6s)
- [PASS] `net.minecraft.network.chat.SubStringSource` -> `mc-protocol` (624.6s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.templatesystem.BlockAgeProcessor` -> `mc-worldgen` (563.4s)
