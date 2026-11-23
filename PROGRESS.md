# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 02:28:20 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 5742 / 66676 (8.612%)
- **Mapped Java Classes**: 1305 / 7163 (18.219%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 2009 mapped methods
- `mc-nbt`: 1541 mapped methods
- `mc-protocol`: 2220 mapped methods
- `mc-worldgen`: 1404 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetContainerContents` -> `mc-nbt` (1266.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.treedecorators.AttachedToLeavesDecorator` -> `mc-worldgen` (276.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.EndPodiumFeature` -> `mc-worldgen` (609.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.TwistingVinesFeature` -> `mc-worldgen` (609.1s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (1915.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket` -> `mc-protocol` (215.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket` -> `mc-protocol` (215.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBlockEventPacket` -> `mc-protocol` (215.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundExplodePacket` -> `mc-protocol` (215.3s)
- [PASS] `net.minecraft.world.level.levelgen.OreVeinifier` -> `mc-worldgen` (623.3s)
- [PASS] `net.minecraft.world.level.levelgen.WorldOptions` -> `mc-worldgen` (623.3s)
- [PASS] `net.minecraft.network.protocol.login.ClientboundHelloPacket` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundKeyPacket` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.login.ClientboundCustomQueryPacket` -> `mc-protocol` (484.8s)
- [PASS] `net.minecraft.network.protocol.login.LoginPacketTypes` -> `mc-protocol` (484.8s)
