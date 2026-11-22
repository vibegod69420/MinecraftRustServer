# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:14:34 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 542 / 66676 (0.813%)
- **Mapped Java Classes**: 182 / 7163 (2.541%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 375 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 123 mapped methods
- `mc-worldgen`: 155 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.RegistryFriendlyByteBuf` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.BandwidthDebugMonitor` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.CipherDecoder` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.CipherEncoder` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.SkipPacketEncoderException` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.SkipPacketDecoderException` -> `mc-protocol` (213.3s)
- [PASS] `net.minecraft.network.chat.contents.TranslatableFormatException` -> `mc-protocol` (210.4s)
- [PASS] `net.minecraft.network.chat.contents.KeybindResolver` -> `mc-protocol` (210.4s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendLoot` -> `mc-worldgen` (196.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.AppendStatic` -> `mc-worldgen` (196.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifierType` -> `mc-worldgen` (196.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Passthrough` -> `mc-worldgen` (196.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.Clear` -> `mc-worldgen` (196.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.rule.blockentity.RuleBlockEntityModifier` -> `mc-worldgen` (196.0s)
- [PASS] `net.minecraft.util.valueproviders.TrapezoidFloat` -> `mc-math` (192.8s)
