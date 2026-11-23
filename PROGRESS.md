# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:26:40 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3696 / 66676 (5.543%)
- **Mapped Java Classes**: 927 / 7163 (12.941%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1299 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1102 mapped methods
- `mc-worldgen`: 953 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.world.level.levelgen.feature.BlockBlobFeature` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ConfiguredFeature` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.CoralMushroomFeature` -> `mc-worldgen` (2169.4s)
- [PASS] `net.minecraft.util.ClassTreeIdRegistry` -> `mc-math` (449.0s)
- [PASS] `net.minecraft.util.TriState` -> `mc-math` (449.0s)
