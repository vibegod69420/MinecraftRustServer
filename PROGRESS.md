# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:10:25 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5438 / 66676 (8.156%)
- **Mapped Java Classes**: 1267 / 7163 (17.688%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1924 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 2030 mapped methods
- `mc-worldgen`: 1338 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.network.PacketDecoder` -> `mc-protocol` (978.4s)
- [PASS] `net.minecraft.network.PacketEncoder` -> `mc-protocol` (978.4s)
- [FAIL] `net.minecraft.util.thread.AbstractConsecutiveExecutor` -> `mc-math` (961.0s)
