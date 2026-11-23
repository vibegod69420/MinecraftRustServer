# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:50:09 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4203 / 66676 (6.304%)
- **Mapped Java Classes**: 1025 / 7163 (14.310%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1491 mapped methods
- `mc-nbt`: 1200 mapped methods
- `mc-protocol`: 1399 mapped methods
- `mc-worldgen`: 1123 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSelectAdvancementsTabPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundCommandSuggestionPacket` -> `mc-protocol` (575.3s)
- [PASS] `net.minecraft.world.level.levelgen.presets.WorldPreset` -> `mc-worldgen` (244.7s)
- [FAIL] `net.minecraft.core.MappedRegistry` -> `mc-math` (1185.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.SpringFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ReplaceBlobsFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockPileFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.DiskFeature` -> `mc-worldgen` (1126.5s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CaveCarverConfiguration` -> `mc-worldgen` (439.7s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CanyonCarverConfiguration` -> `mc-worldgen` (439.7s)
- [PASS] `net.minecraft.world.level.levelgen.carver.CarverDebugSettings` -> `mc-worldgen` (439.7s)
- [FAIL] `net.minecraft.util.ByIdMap` -> `mc-math` (595.3s)
- [PASS] `net.minecraft.util.FormattedCharSequence` -> `mc-math` (595.3s)
