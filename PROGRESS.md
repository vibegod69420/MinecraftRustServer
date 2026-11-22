# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:04:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 233 / 66676 (0.349%)
- **Mapped Java Classes**: 77 / 7163 (1.075%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 238 mapped methods
- `mc-nbt`: 8 mapped methods
- `mc-protocol`: 24 mapped methods
- `mc-worldgen`: 42 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.CountConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.ProbabilityFeatureConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.BlockPileConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.BlockStateConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.NoneFeatureConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.FeatureConfiguration` -> `mc-worldgen` (209.6s)
- [PASS] `net.minecraft.network.protocol.ping.ClientboundPongResponsePacket` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.PingPacketTypes` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.ClientPongPacketListener` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.network.protocol.ping.ServerPingPacketListener` -> `mc-protocol` (207.0s)
- [PASS] `net.minecraft.world.level.levelgen.material.WorldGenMaterialRule` -> `mc-worldgen` (163.9s)
- [PASS] `net.minecraft.world.level.levelgen.material.MaterialRuleList` -> `mc-worldgen` (163.9s)
- [PASS] `net.minecraft.network.codec.StreamDecoder` -> `mc-protocol` (121.3s)
- [PASS] `net.minecraft.util.random.WeightedRandom` -> `mc-math` (737.8s)
- [PASS] `net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload` -> `mc-protocol` (157.3s)
