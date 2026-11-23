# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:18:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5592 / 66676 (8.387%)
- **Mapped Java Classes**: 1285 / 7163 (17.939%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2061 mapped methods
- `mc-worldgen`: 1369 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.network.chat.SignedMessageLink` -> `mc-protocol` (971.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.OceanMonumentStructure` -> `mc-worldgen` (483.5s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.DesertPyramidStructure` -> `mc-worldgen` (483.5s)
