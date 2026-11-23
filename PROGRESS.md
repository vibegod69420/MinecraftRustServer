# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:27:58 UTC
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
- [PASS] `net.minecraft.world.level.levelgen.feature.ScatteredOreFeature` -> `mc-worldgen` (1176.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.EndGatewayFeature` -> `mc-worldgen` (1176.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.NetherForestVegetationFeature` -> `mc-worldgen` (1176.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.GlowstoneFeature` -> `mc-worldgen` (1176.7s)
- [FAIL] `net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket` -> `mc-protocol` (728.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.VeryBiasedToBottomHeight` -> `mc-worldgen` (582.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.BiasedToBottomHeight` -> `mc-worldgen` (582.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.UniformHeight` -> `mc-worldgen` (582.6s)
- [FAIL] `net.minecraft.util.BoundedFloatFunction` -> `mc-math` (529.3s)
- [PASS] `net.minecraft.util.FutureChain` -> `mc-math` (529.3s)
- [PASS] `net.minecraft.util.ZeroBitStorage` -> `mc-math` (529.3s)
- [PASS] `net.minecraft.util.StrictJsonParser` -> `mc-math` (529.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetContainerLootTable` -> `mc-nbt` (402.0s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetBannerPatternFunction` -> `mc-nbt` (402.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.HugeFungusConfiguration` -> `mc-worldgen` (2169.4s)
