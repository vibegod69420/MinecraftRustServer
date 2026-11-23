# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:06:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3233 / 66676 (4.849%)
- **Mapped Java Classes**: 800 / 7163 (11.168%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1178 mapped methods
- `mc-nbt`: 1092 mapped methods
- `mc-protocol`: 876 mapped methods
- `mc-worldgen`: 768 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.context.ContextMap` -> `mc-math` (308.4s)
- [PASS] `net.minecraft.util.context.ContextKeySet` -> `mc-math` (308.4s)
- [PASS] `net.minecraft.util.context.ContextKey` -> `mc-math` (308.4s)
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
