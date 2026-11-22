# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:00:10 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2105 / 66676 (3.157%)
- **Mapped Java Classes**: 576 / 7163 (8.041%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 814 mapped methods
- `mc-nbt`: 614 mapped methods
- `mc-protocol`: 528 mapped methods
- `mc-worldgen`: 565 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.RandomGroupPoolAlias` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasLookup` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBindings` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket` -> `mc-protocol` (271.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPickItemFromBlockPacket` -> `mc-protocol` (271.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket` -> `mc-protocol` (271.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundGameTestHighlightPosPacket` -> `mc-protocol` (271.3s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (651.8s)
- [PASS] `net.minecraft.core.component.DataComponentMap` -> `mc-math` (299.3s)
- [PASS] `net.minecraft.core.dispenser.ShearsDispenseItemBehavior` -> `mc-math` (1392.7s)
- [PASS] `net.minecraft.core.dispenser.BoatDispenseItemBehavior` -> `mc-math` (1392.7s)
- [PASS] `net.minecraft.core.dispenser.ProjectileDispenseBehavior` -> `mc-math` (1392.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.WeightedPlacedFeature` -> `mc-worldgen` (686.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.EndIslandFeature` -> `mc-worldgen` (686.2s)
