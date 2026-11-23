# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:43:44 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5955 / 66676 (8.931%)
- **Mapped Java Classes**: 1340 / 7163 (18.707%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2322 mapped methods
- `mc-worldgen`: 1539 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor` -> `mc-worldgen` (250.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.CherryFoliagePlacer` -> `mc-worldgen` (571.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BasaltPillarFeature` -> `mc-worldgen` (746.8s)
- [FAIL] `net.minecraft.network.protocol.game.DebugEntityNameGenerator` -> `mc-protocol` (975.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundMapItemDataPacket` -> `mc-protocol` (975.7s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetCommandMinecartPacket` -> `mc-protocol` (975.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRecipeBookAddPacket` -> `mc-protocol` (975.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacer` -> `mc-worldgen` (441.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.PlaceOnGroundDecorator` -> `mc-worldgen` (569.1s)
- [PASS] `net.minecraft.world.level.storage.PrimaryLevelData` -> `mc-nbt` (1479.5s)
- [FAIL] `net.minecraft.network.chat.contents.data.BlockDataSource` -> `mc-protocol` (486.8s)
- [PASS] `net.minecraft.network.chat.contents.data.EntityDataSource` -> `mc-protocol` (486.8s)
- [PASS] `net.minecraft.world.level.levelgen.feature.WeepingVinesFeature` -> `mc-worldgen` (357.5s)
- [PASS] `net.minecraft.network.protocol.BundlerInfo` -> `mc-protocol` (900.5s)
- [PASS] `net.minecraft.network.protocol.PacketUtils` -> `mc-protocol` (900.5s)
