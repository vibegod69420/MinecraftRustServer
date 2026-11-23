# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:45:52 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 4104 / 66676 (6.155%)
- **Mapped Java Classes**: 1003 / 7163 (14.002%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1478 mapped methods
- `mc-nbt`: 1191 mapped methods
- `mc-protocol`: 1333 mapped methods
- `mc-worldgen`: 1064 mapped methods

## Recent Verified Work
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
- [PASS] `net.minecraft.network.protocol.game.VecDeltaCodec` -> `mc-protocol` (313.5s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundTeleportToEntityPacket` -> `mc-protocol` (313.5s)
