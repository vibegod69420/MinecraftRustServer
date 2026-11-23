# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 03:32:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 6344 / 66676 (9.515%)
- **Mapped Java Classes**: 1397 / 7163 (19.503%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2730 mapped methods
- `mc-worldgen`: 1624 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundLoginPacket` -> `mc-protocol` (602.9s)
- [PASS] `net.minecraft.network.protocol.game.DebugEntityNameGenerator` -> `mc-protocol` (602.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket` -> `mc-protocol` (602.9s)
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
