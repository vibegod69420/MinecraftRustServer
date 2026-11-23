# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 01:10:27 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4591 / 66676 (6.886%)
- **Mapped Java Classes**: 1109 / 7163 (15.482%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1597 mapped methods
- `mc-nbt`: 1347 mapped methods
- `mc-protocol`: 1518 mapped methods
- `mc-worldgen`: 1253 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetCustomModelDataFunction` -> `mc-nbt` (466.3s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.NetherFossilStructure` -> `mc-worldgen` (454.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure` -> `mc-worldgen` (454.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure` -> `mc-worldgen` (454.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction` -> `mc-nbt` (559.6s)
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
