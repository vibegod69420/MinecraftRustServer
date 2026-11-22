# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:25:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1017 / 66676 (1.525%)
- **Mapped Java Classes**: 316 / 7163 (4.412%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 532 mapped methods
- `mc-nbt`: 193 mapped methods
- `mc-protocol`: 213 mapped methods
- `mc-worldgen`: 315 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.TagParser` -> `mc-nbt` (225.3s)
- [PASS] `net.minecraft.nbt.ByteTag` -> `mc-nbt` (225.3s)
- [FAIL] `net.minecraft.nbt.LongTag` -> `mc-nbt` (397.9s)
- [FAIL] `net.minecraft.nbt.ShortTag` -> `mc-nbt` (397.9s)
- [FAIL] `net.minecraft.nbt.IntTag` -> `mc-nbt` (397.9s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockMatchTest` -> `mc-worldgen` (154.7s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.RandomBlockStateMatchTest` -> `mc-worldgen` (154.7s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.RuleTestType` -> `mc-worldgen` (154.7s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTestType` -> `mc-worldgen` (154.7s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBundlePacket` -> `mc-protocol` (368.2s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundBundleDelimiterPacket` -> `mc-protocol` (368.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket` -> `mc-protocol` (368.2s)
- [PASS] `net.minecraft.network.protocol.game.ServerPacketListener` -> `mc-protocol` (368.2s)
- [PASS] `net.minecraft.network.chat.contents.data.DataSources` -> `mc-protocol` (119.8s)
- [PASS] `net.minecraft.nbt.visitors.CollectFields` -> `mc-nbt` (347.0s)
