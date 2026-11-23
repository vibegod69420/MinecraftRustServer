# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:43:38 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4028 / 66676 (6.041%)
- **Mapped Java Classes**: 992 / 7163 (13.849%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1430 mapped methods
- `mc-nbt`: 1173 mapped methods
- `mc-protocol`: 1324 mapped methods
- `mc-worldgen`: 1052 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.LevelData` -> `mc-nbt` (790.1s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (790.1s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (790.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket` -> `mc-protocol` (313.5s)
- [PASS] `net.minecraft.network.protocol.game.VecDeltaCodec` -> `mc-protocol` (313.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket` -> `mc-protocol` (313.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundClientCommandPacket` -> `mc-protocol` (313.5s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket` -> `mc-protocol` (653.7s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket` -> `mc-protocol` (653.7s)
- [PASS] `net.minecraft.network.protocol.configuration.ConfigurationPacketTypes` -> `mc-protocol` (653.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration` -> `mc-worldgen` (211.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.LargeDripstoneConfiguration` -> `mc-worldgen` (211.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration` -> `mc-worldgen` (211.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRespawnPacket` -> `mc-protocol` (622.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket` -> `mc-protocol` (622.2s)
