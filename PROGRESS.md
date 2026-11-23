# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:21:52 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5667 / 66676 (8.499%)
- **Mapped Java Classes**: 1296 / 7163 (18.093%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2138 mapped methods
- `mc-worldgen`: 1391 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.login.ClientboundHelloPacket` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundKeyPacket` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.login.ClientboundCustomQueryPacket` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.login.LoginPacketTypes` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.common.CommonPacketTypes` -> `mc-protocol` (224.0s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket` -> `mc-protocol` (224.0s)
- [FAIL] `net.minecraft.world.level.storage.PlayerDataStorage` -> `mc-nbt` (1073.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction` -> `mc-nbt` (439.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.RandomizedIntStateProvider` -> `mc-worldgen` (765.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.JigsawJunction` -> `mc-worldgen` (172.2s)
- [PASS] `net.minecraft.util.ArrayListDeque` -> `mc-math` (1309.6s)
- [FAIL] `net.minecraft.util.GsonHelper` -> `mc-math` (773.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DeltaFeature` -> `mc-worldgen` (1534.5s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.MultifaceGrowthFeature` -> `mc-worldgen` (1534.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.WaterloggedVegetationPatchFeature` -> `mc-worldgen` (1534.5s)
