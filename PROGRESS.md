# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:48:57 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4178 / 66676 (6.266%)
- **Mapped Java Classes**: 1020 / 7163 (14.240%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1486 mapped methods
- `mc-nbt`: 1200 mapped methods
- `mc-protocol`: 1369 mapped methods
- `mc-worldgen`: 1118 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.storage.loot.ContainerComponentManipulators` -> `mc-nbt` (254.1s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.EntityHasScoreCondition` -> `mc-nbt` (318.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket` -> `mc-protocol` (594.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundCustomChatCompletionsPacket` -> `mc-protocol` (594.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket` -> `mc-protocol` (594.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderWarningDelayPacket` -> `mc-protocol` (594.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SnowAndFreezeFeature` -> `mc-worldgen` (848.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.CoralTreeFeature` -> `mc-worldgen` (848.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SeaPickleFeature` -> `mc-worldgen` (848.2s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SimpleBlockFeature` -> `mc-worldgen` (848.2s)
- [PASS] `net.minecraft.core.component.PatchedDataComponentMap` -> `mc-math` (820.1s)
- [FAIL] `net.minecraft.world.level.storage.LevelData` -> `mc-nbt` (790.1s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (790.1s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (790.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetBorderWarningDistancePacket` -> `mc-protocol` (313.5s)
