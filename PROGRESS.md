# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:14:32 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3425 / 66676 (5.137%)
- **Mapped Java Classes**: 861 / 7163 (12.020%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1235 mapped methods
- `mc-nbt`: 1113 mapped methods
- `mc-protocol`: 952 mapped methods
- `mc-worldgen`: 858 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor` -> `mc-worldgen` (363.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor` -> `mc-worldgen` (363.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.AxisAlignedLinearPosTest` -> `mc-worldgen` (363.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.GravityProcessor` -> `mc-worldgen` (363.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (363.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (363.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (363.1s)
- [PASS] `net.minecraft.util.debug.DebugGoalInfo` -> `mc-math` (349.4s)
- [PASS] `net.minecraft.util.debug.DebugGameEventInfo` -> `mc-math` (349.4s)
- [PASS] `net.minecraft.util.debug.DebugValueAccess` -> `mc-math` (349.4s)
- [PASS] `net.minecraft.util.debug.DebugEntityBlockIntersection` -> `mc-math` (349.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket` -> `mc-protocol` (337.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundDisguisedChatPacket` -> `mc-protocol` (337.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket` -> `mc-protocol` (337.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundContainerClosePacket` -> `mc-protocol` (337.3s)
