# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:54:23 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3100 / 66676 (4.649%)
- **Mapped Java Classes**: 771 / 7163 (10.764%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1098 mapped methods
- `mc-nbt`: 1032 mapped methods
- `mc-protocol`: 842 mapped methods
- `mc-worldgen`: 755 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer` -> `mc-worldgen` (827.7s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.foliageplacers.FancyFoliagePlacer` -> `mc-worldgen` (827.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.MegaJungleFoliagePlacer` -> `mc-worldgen` (827.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.BushFoliagePlacer` -> `mc-worldgen` (827.7s)
- [FAIL] `net.minecraft.core.MappedRegistry` -> `mc-math` (907.3s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.MegaJungleTrunkPlacer` -> `mc-worldgen` (683.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.TrunkPlacerType` -> `mc-worldgen` (683.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.trunkplacers.StraightTrunkPlacer` -> `mc-worldgen` (683.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSelectTradePacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundRenameItemPacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugEntityValuePacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugChunkValuePacket` -> `mc-protocol` (519.0s)
- [PASS] `net.minecraft.world.level.storage.loot.parameters.LootContextParams` -> `mc-nbt` (112.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetCarriedItemPacket` -> `mc-protocol` (318.8s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundResetScorePacket` -> `mc-protocol` (318.8s)
