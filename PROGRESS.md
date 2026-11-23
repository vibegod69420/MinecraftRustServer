# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:41:38 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4010 / 66676 (6.014%)
- **Mapped Java Classes**: 988 / 7163 (13.793%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1430 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1324 mapped methods
- `mc-worldgen`: 1042 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket` -> `mc-protocol` (653.7s)
- [PASS] `net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket` -> `mc-protocol` (653.7s)
- [PASS] `net.minecraft.network.protocol.configuration.ConfigurationPacketTypes` -> `mc-protocol` (653.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.FallenTreeConfiguration` -> `mc-worldgen` (211.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.LargeDripstoneConfiguration` -> `mc-worldgen` (211.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.BlockColumnConfiguration` -> `mc-worldgen` (211.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRespawnPacket` -> `mc-protocol` (622.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRemoveMobEffectPacket` -> `mc-protocol` (622.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundContainerSlotStateChangedPacket` -> `mc-protocol` (622.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundEditBookPacket` -> `mc-protocol` (622.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (334.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer` -> `mc-nbt` (334.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.TagEntry` -> `mc-nbt` (334.7s)
- [PASS] `net.minecraft.util.SignatureValidator` -> `mc-math` (594.8s)
- [PASS] `net.minecraft.util.CsvOutput` -> `mc-math` (594.8s)
