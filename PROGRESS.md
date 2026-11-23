# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:29:19 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3725 / 66676 (5.587%)
- **Mapped Java Classes**: 934 / 7163 (13.039%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1320 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1102 mapped methods
- `mc-worldgen`: 961 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.util.IdentifierPattern` -> `mc-math` (738.8s)
- [PASS] `net.minecraft.util.AbstractListBuilder` -> `mc-math` (738.8s)
- [PASS] `net.minecraft.util.SegmentedAnglePrecision` -> `mc-math` (738.8s)
- [PASS] `net.minecraft.util.ModCheck` -> `mc-math` (738.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BlockPileFeature` -> `mc-worldgen` (874.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DiskFeature` -> `mc-worldgen` (874.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (874.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.VoidStartPlatformFeature` -> `mc-worldgen` (874.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ScatteredOreFeature` -> `mc-worldgen` (1176.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.EndGatewayFeature` -> `mc-worldgen` (1176.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.NetherForestVegetationFeature` -> `mc-worldgen` (1176.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.GlowstoneFeature` -> `mc-worldgen` (1176.7s)
- [FAIL] `net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket` -> `mc-protocol` (728.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight` -> `mc-worldgen` (582.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight` -> `mc-worldgen` (582.6s)
