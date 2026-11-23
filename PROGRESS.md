# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:14:15 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5493 / 66676 (8.238%)
- **Mapped Java Classes**: 1278 / 7163 (17.842%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1939 mapped methods
- `mc-nbt`: 1530 mapped methods
- `mc-protocol`: 2053 mapped methods
- `mc-worldgen`: 1355 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyBlockState` -> `mc-nbt` (133.4s)
- [PASS] `net.minecraft.util.PlaceholderLookupProvider` -> `mc-math` (1371.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.WoodlandMansionStructure` -> `mc-worldgen` (1304.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.OceanRuinStructure` -> `mc-worldgen` (1304.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure` -> `mc-worldgen` (1304.6s)
- [FAIL] `net.minecraft.util.HashOps` -> `mc-math` (837.3s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SculkPatchFeature` -> `mc-worldgen` (924.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.CoralFeature` -> `mc-worldgen` (924.8s)
- [PASS] `net.minecraft.util.ExtraCodecs` -> `mc-math` (124.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundLevelEventPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundUseItemPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundOpenScreenPacket` -> `mc-protocol` (707.8s)
- [PASS] `net.minecraft.network.chat.contents.KeybindContents` -> `mc-protocol` (1046.3s)
- [PASS] `net.minecraft.network.chat.contents.SelectorContents` -> `mc-protocol` (1046.3s)
