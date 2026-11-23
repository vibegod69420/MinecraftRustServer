# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:33:35 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3745 / 66676 (5.617%)
- **Mapped Java Classes**: 938 / 7163 (13.095%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1320 mapped methods
- `mc-nbt`: 1157 mapped methods
- `mc-protocol`: 1125 mapped methods
- `mc-worldgen`: 961 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundAwardStatsPacket` -> `mc-protocol` (203.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerCombatKillPacket` -> `mc-protocol` (203.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundContainerClosePacket` -> `mc-protocol` (203.7s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundSetBeaconPacket` -> `mc-protocol` (203.7s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.ShipwreckStructure` -> `mc-worldgen` (909.0s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.structures.NetherFortressStructure` -> `mc-worldgen` (909.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.structures.StrongholdStructure` -> `mc-worldgen` (909.0s)
- [FAIL] `net.minecraft.util.IdentifierPattern` -> `mc-math` (738.8s)
- [PASS] `net.minecraft.util.AbstractListBuilder` -> `mc-math` (738.8s)
- [PASS] `net.minecraft.util.SegmentedAnglePrecision` -> `mc-math` (738.8s)
- [PASS] `net.minecraft.util.ModCheck` -> `mc-math` (738.8s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.BlockPileFeature` -> `mc-worldgen` (874.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.DiskFeature` -> `mc-worldgen` (874.9s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (874.9s)
- [PASS] `net.minecraft.world.level.levelgen.feature.VoidStartPlatformFeature` -> `mc-worldgen` (874.9s)
