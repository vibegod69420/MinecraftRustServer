# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:40:07 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1500 / 66676 (2.250%)
- **Mapped Java Classes**: 441 / 7163 (6.157%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 591 mapped methods
- `mc-nbt`: 455 mapped methods
- `mc-protocol`: 309 mapped methods
- `mc-worldgen`: 446 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.InsideWorldBoundsPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.UnobstructedPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.MatchingBlockTagPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.NotPredicate` -> `mc-worldgen` (943.5s)
- [PASS] `net.minecraft.world.level.levelgen.placement.HeightmapPlacement` -> `mc-worldgen` (545.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.BlockPredicateFilter` -> `mc-worldgen` (545.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.BiomeFilter` -> `mc-worldgen` (545.6s)
- [PASS] `net.minecraft.world.level.levelgen.placement.CountPlacement` -> `mc-worldgen` (545.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.DimensionPadding` -> `mc-worldgen` (155.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.StructurePoolElementType` -> `mc-worldgen` (155.2s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.core.particles.ColorParticleOption` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.core.particles.BlockParticleOption` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.core.particles.DustParticleOptions` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootPoolEntry` -> `mc-nbt` (313.8s)
