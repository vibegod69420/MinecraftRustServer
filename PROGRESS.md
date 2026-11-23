# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:31:19 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6307 / 66676 (9.459%)
- **Mapped Java Classes**: 1392 / 7163 (19.433%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2702 mapped methods
- `mc-worldgen`: 1612 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundCommandSuggestionsPacket` -> `mc-protocol` (503.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket` -> `mc-protocol` (503.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSoundPacket` -> `mc-protocol` (503.1s)
- [FAIL] `net.minecraft.util.GsonHelper` -> `mc-math` (501.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (445.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DeltaFeature` -> `mc-worldgen` (445.1s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature` -> `mc-worldgen` (445.1s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (421.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerChatPacket` -> `mc-protocol` (388.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket` -> `mc-protocol` (388.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket` -> `mc-protocol` (388.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.VegetationPatchFeature` -> `mc-worldgen` (2129.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltColumnsFeature` -> `mc-worldgen` (1180.9s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CanyonWorldCarver` -> `mc-worldgen` (1352.5s)
- [PASS] `net.minecraft.network.chat.FormattedText` -> `mc-protocol` (720.0s)
