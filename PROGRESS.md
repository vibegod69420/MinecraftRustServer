# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:37:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3832 / 66676 (5.747%)
- **Mapped Java Classes**: 954 / 7163 (13.318%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1380 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1198 mapped methods
- `mc-worldgen`: 961 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.dispenser.DispenseItemBehavior` -> `mc-math` (411.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (373.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (373.4s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (373.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket` -> `mc-protocol` (338.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugSamplePacket` -> `mc-protocol` (338.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDebugBlockValuePacket` -> `mc-protocol` (338.9s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundTabListPacket` -> `mc-protocol` (338.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.FunctionReference` -> `mc-nbt` (338.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworkExplosionFunction` -> `mc-nbt` (338.8s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetItemDamageFunction` -> `mc-nbt` (338.8s)
- [PASS] `net.minecraft.util.thread.StrictQueue` -> `mc-math` (315.3s)
- [PASS] `net.minecraft.util.thread.PriorityConsecutiveExecutor` -> `mc-math` (315.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (291.9s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (291.9s)
