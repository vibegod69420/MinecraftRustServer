# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:04:49 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2203 / 66676 (3.304%)
- **Mapped Java Classes**: 583 / 7163 (8.139%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 938 mapped methods
- `mc-nbt`: 614 mapped methods
- `mc-protocol`: 528 mapped methods
- `mc-worldgen`: 565 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.UUIDUtil` -> `mc-math` (1133.3s)
- [FAIL] `net.minecraft.nbt.NbtUtils` -> `mc-nbt` (2073.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.AllOfCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.InvertedLootItemCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.ExplosionCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.core.Vec3i` -> `mc-math` (485.1s)
- [PASS] `net.minecraft.core.HolderLookup` -> `mc-math` (711.3s)
- [FAIL] `net.minecraft.core.component.predicates.DataComponentPredicate` -> `mc-math` (547.7s)
- [PASS] `net.minecraft.core.Holder` -> `mc-math` (342.6s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.RandomGroupPoolAlias` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasLookup` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.DirectPoolAlias` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pools.alias.PoolAliasBindings` -> `mc-worldgen` (394.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket` -> `mc-protocol` (271.3s)
