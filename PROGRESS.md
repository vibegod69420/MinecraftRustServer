# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:08:11 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4567 / 66676 (6.849%)
- **Mapped Java Classes**: 1104 / 7163 (15.412%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1597 mapped methods
- `mc-nbt`: 1321 mapped methods
- `mc-protocol`: 1518 mapped methods
- `mc-worldgen`: 1241 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.functions.EnchantRandomlyFunction` -> `mc-nbt` (286.5s)
- [FAIL] `net.minecraft.util.thread.AbstractConsecutiveExecutor` -> `mc-math` (685.2s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetNameFunction` -> `mc-nbt` (181.1s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.CappedProcessor` -> `mc-worldgen` (641.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.StructureProcessorType` -> `mc-worldgen` (641.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.JigsawReplacementProcessor` -> `mc-worldgen` (641.0s)
- [PASS] `net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap` -> `mc-math` (230.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundEntityEventPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderCenterPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundUpdateRecipesPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetEntityLinkPacket` -> `mc-protocol` (299.8s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction` -> `mc-nbt` (177.4s)
- [FAIL] `net.minecraft.world.level.storage.PlayerDataStorage` -> `mc-nbt` (1294.9s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundOpenSignEditorPacket` -> `mc-protocol` (773.1s)
- [FAIL] `net.minecraft.network.protocol.game.ServerboundBlockEntityTagQueryPacket` -> `mc-protocol` (773.1s)
