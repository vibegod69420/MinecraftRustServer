# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:16:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3441 / 66676 (5.161%)
- **Mapped Java Classes**: 865 / 7163 (12.076%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1235 mapped methods
- `mc-nbt`: 1113 mapped methods
- `mc-protocol`: 968 mapped methods
- `mc-worldgen`: 858 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.status.ClientboundStatusResponsePacket` -> `mc-protocol` (232.1s)
- [PASS] `net.minecraft.network.CompressionEncoder` -> `mc-protocol` (531.4s)
- [PASS] `net.minecraft.network.CipherBase` -> `mc-protocol` (531.4s)
- [PASS] `net.minecraft.network.VarLong` -> `mc-protocol` (531.4s)
- [PASS] `net.minecraft.network.PacketListener` -> `mc-protocol` (531.4s)
- [PASS] `net.minecraft.util.debug.DebugHiveInfo` -> `mc-math` (189.9s)
- [PASS] `net.minecraft.util.debug.DebugStructureInfo` -> `mc-math` (189.9s)
- [PASS] `net.minecraft.util.debug.DebugPoiInfo` -> `mc-math` (189.9s)
- [PASS] `net.minecraft.util.debug.DebugBeeInfo` -> `mc-math` (189.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.BlockRotProcessor` -> `mc-worldgen` (363.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.BlockIgnoreProcessor` -> `mc-worldgen` (363.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.AxisAlignedLinearPosTest` -> `mc-worldgen` (363.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.GravityProcessor` -> `mc-worldgen` (363.5s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (363.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (363.1s)
