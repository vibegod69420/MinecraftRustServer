# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:01:07 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5282 / 66676 (7.922%)
- **Mapped Java Classes**: 1239 / 7163 (17.297%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1868 mapped methods
- `mc-nbt`: 1524 mapped methods
- `mc-protocol`: 1926 mapped methods
- `mc-worldgen`: 1298 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.DripstoneClusterConfiguration` -> `mc-worldgen` (172.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.VegetationPatchConfiguration` -> `mc-worldgen` (172.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.TreeDecorator` -> `mc-worldgen` (558.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.BeehiveDecorator` -> `mc-worldgen` (558.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.treedecorators.AlterGroundDecorator` -> `mc-worldgen` (558.7s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (524.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece` -> `mc-worldgen` (299.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.BuiltinStructures` -> `mc-worldgen` (299.4s)
- [PASS] `net.minecraft.util.DirectoryLock` -> `mc-math` (439.9s)
- [PASS] `net.minecraft.util.FileUtil` -> `mc-math` (439.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetPassengersPacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundJigsawGeneratePacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockDestructionPacket` -> `mc-protocol` (388.5s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetExperiencePacket` -> `mc-protocol` (325.8s)
