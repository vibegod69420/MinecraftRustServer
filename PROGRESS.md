# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:39:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3898 / 66676 (5.846%)
- **Mapped Java Classes**: 964 / 7163 (13.458%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1424 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1198 mapped methods
- `mc-worldgen`: 1009 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.SpruceFoliagePlacer` -> `mc-worldgen` (531.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.PineFoliagePlacer` -> `mc-worldgen` (531.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.AcaciaFoliagePlacer` -> `mc-worldgen` (531.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.foliageplacers.FancyFoliagePlacer` -> `mc-worldgen` (531.0s)
- [PASS] `net.minecraft.util.debug.DebugSubscription` -> `mc-math` (440.4s)
- [PASS] `net.minecraft.util.debug.ServerDebugSubscribers` -> `mc-math` (440.4s)
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (411.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (373.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (373.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (373.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket` -> `mc-protocol` (338.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugSamplePacket` -> `mc-protocol` (338.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket` -> `mc-protocol` (338.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTabListPacket` -> `mc-protocol` (338.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (338.8s)
