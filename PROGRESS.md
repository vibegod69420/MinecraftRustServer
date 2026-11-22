# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:45:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2857 / 66676 (4.285%)
- **Mapped Java Classes**: 720 / 7163 (10.052%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1058 mapped methods
- `mc-nbt`: 954 mapped methods
- `mc-protocol`: 690 mapped methods
- `mc-worldgen`: 701 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.component.DataComponents` -> `mc-math` (290.7s)
- [PASS] `net.minecraft.network.chat.contents.objects.AtlasSprite` -> `mc-protocol` (111.3s)
- [PASS] `net.minecraft.network.chat.contents.objects.PlayerSprite` -> `mc-protocol` (111.3s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (1057.1s)
- [PASS] `net.minecraft.world.level.storage.LevelVersion` -> `mc-nbt` (1057.1s)
- [PASS] `net.minecraft.world.level.storage.ValueInput` -> `mc-nbt` (1057.1s)
- [PASS] `net.minecraft.world.level.storage.LevelResource` -> `mc-nbt` (1057.1s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket` -> `mc-protocol` (320.7s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundClientInformationPacket` -> `mc-protocol` (320.7s)
- [PASS] `net.minecraft.network.protocol.common.ServerboundKeepAlivePacket` -> `mc-protocol` (320.7s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundKeepAlivePacket` -> `mc-protocol` (320.7s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.CopyNameFunction` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ToggleTooltips` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ApplyExplosionDecay` -> `mc-nbt` (241.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetCustomDataFunction` -> `mc-nbt` (241.6s)
