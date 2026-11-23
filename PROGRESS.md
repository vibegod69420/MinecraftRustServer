# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:46:16 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5978 / 66676 (8.966%)
- **Mapped Java Classes**: 1344 / 7163 (18.763%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1604 mapped methods
- `mc-protocol`: 2354 mapped methods
- `mc-worldgen`: 1539 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.network.protocol.game.ClientboundBlockEntityDataPacket` -> `mc-protocol` (774.8s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundContainerClickPacket` -> `mc-protocol` (774.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundStopSoundPacket` -> `mc-protocol` (774.8s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (2646.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.PaleMossDecorator` -> `mc-worldgen` (801.1s)
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
