# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:04:27 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 253 / 66676 (0.379%)
- **Mapped Java Classes**: 86 / 7163 (1.201%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 238 mapped methods
- `mc-nbt`: 8 mapped methods
- `mc-protocol`: 36 mapped methods
- `mc-worldgen`: 55 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.AlwaysTrueTest` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTest` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.LiquidSettings` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.RuleTest` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.NopProcessor` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorList` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.CountConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.ProbabilityFeatureConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.BlockPileConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.BlockStateConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.network.protocol.ping.ClientboundPongResponsePacket` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.PingPacketTypes` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.ClientPongPacketListener` -> `mc-protocol` (207.0s)
