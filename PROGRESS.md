# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:32:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2299 / 66676 (3.448%)
- **Mapped Java Classes**: 592 / 7163 (8.265%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 944 mapped methods
- `mc-nbt`: 679 mapped methods
- `mc-protocol`: 559 mapped methods
- `mc-worldgen`: 569 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket` -> `mc-protocol` (239.7s)
- [PASS] `net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket` -> `mc-protocol` (239.7s)
- [PASS] `net.minecraft.nbt.ListTag` -> `mc-nbt` (181.4s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (123.8s)
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType` -> `mc-worldgen` (113.5s)
- [FAIL] `net.minecraft.core.Registry` -> `mc-math` (2076.1s)
- [FAIL] `net.minecraft.core.HolderSet` -> `mc-math` (1186.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition` -> `mc-nbt` (601.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemCondition` -> `mc-nbt` (601.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.MatchTool` -> `mc-nbt` (601.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.EnchantmentActiveCheck` -> `mc-nbt` (601.5s)
- [PASS] `net.minecraft.core.UUIDUtil` -> `mc-math` (1133.3s)
- [FAIL] `net.minecraft.nbt.NbtUtils` -> `mc-nbt` (2073.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.AllOfCondition` -> `mc-nbt` (805.3s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceCondition` -> `mc-nbt` (805.3s)
