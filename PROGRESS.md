# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:11:46 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5458 / 66676 (8.186%)
- **Mapped Java Classes**: 1270 / 7163 (17.730%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1924 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 2053 mapped methods
- `mc-worldgen`: 1338 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (924.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.CoralFeature` -> `mc-worldgen` (924.8s)
- [PASS] `net.minecraft.util.ExtraCodecs` -> `mc-math` (124.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundLevelEventPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundUseItemPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundOpenScreenPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.chat.contents.KeybindContents` -> `mc-protocol` (1046.3s)
- [PASS] `net.minecraft.network.chat.contents.SelectorContents` -> `mc-protocol` (1046.3s)
- [PASS] `net.minecraft.network.chat.contents.PlainTextContents` -> `mc-protocol` (1046.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.HugeRedMushroomFeature` -> `mc-worldgen` (1026.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.PointedDripstoneFeature` -> `mc-worldgen` (1026.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.BonusChestFeature` -> `mc-worldgen` (1026.2s)
- [FAIL] `net.minecraft.network.HashedStack` -> `mc-protocol` (978.4s)
- [FAIL] `net.minecraft.network.PacketProcessor` -> `mc-protocol` (978.4s)
