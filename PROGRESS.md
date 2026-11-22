# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:36:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1362 / 66676 (2.043%)
- **Mapped Java Classes**: 397 / 7163 (5.542%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 588 mapped methods
- `mc-nbt`: 390 mapped methods
- `mc-protocol`: 273 mapped methods
- `mc-worldgen`: 391 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.StringTagVisitor` -> `mc-nbt` (775.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders` -> `mc-nbt` (160.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider` -> `mc-nbt` (160.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider` -> `mc-nbt` (160.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType` -> `mc-nbt` (160.7s)
- [FAIL] `net.minecraft.network.protocol.ProtocolCodecBuilder` -> `mc-protocol` (702.8s)
- [FAIL] `net.minecraft.network.protocol.Packet` -> `mc-protocol` (702.8s)
- [PASS] `net.minecraft.world.level.storage.DataVersion` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.world.level.storage.LevelStorageException` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.world.level.storage.LevelDataAndDimensions` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.world.level.storage.WritableLevelData` -> `mc-nbt` (149.2s)
- [PASS] `net.minecraft.nbt.LongArrayTag` -> `mc-nbt` (529.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.TwoLayersFeatureSize` -> `mc-worldgen` (407.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSize` -> `mc-worldgen` (407.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.featuresize.FeatureSizeType` -> `mc-worldgen` (407.4s)
