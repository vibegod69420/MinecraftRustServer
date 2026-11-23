# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:20:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5642 / 66676 (8.462%)
- **Mapped Java Classes**: 1290 / 7163 (18.009%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2112 mapped methods
- `mc-worldgen`: 1385 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction` -> `mc-nbt` (439.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider` -> `mc-worldgen` (765.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.JigsawJunction` -> `mc-worldgen` (172.2s)
- [PASS] `net.minecraft.util.ArrayListDeque` -> `mc-math` (1309.6s)
- [FAIL] `net.minecraft.util.GsonHelper` -> `mc-math` (773.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DeltaFeature` -> `mc-worldgen` (1534.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature` -> `mc-worldgen` (1534.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature` -> `mc-worldgen` (1534.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSignUpdatePacket` -> `mc-protocol` (597.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundLightUpdatePacket` -> `mc-protocol` (597.6s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundRecipeBookChangeSettingsPacket` -> `mc-protocol` (597.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetDisplayObjectivePacket` -> `mc-protocol` (597.6s)
- [FAIL] `net.minecraft.network.chat.SignableCommand` -> `mc-protocol` (971.1s)
- [PASS] `net.minecraft.network.chat.RemoteChatSession` -> `mc-protocol` (971.1s)
- [PASS] `net.minecraft.network.chat.MessageSignatureCache` -> `mc-protocol` (971.1s)
