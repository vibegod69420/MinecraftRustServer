# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:36:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1403 / 66676 (2.104%)
- **Mapped Java Classes**: 406 / 7163 (5.668%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 588 mapped methods
- `mc-nbt`: 402 mapped methods
- `mc-protocol`: 309 mapped methods
- `mc-worldgen`: 391 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket` -> `mc-protocol` (252.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket` -> `mc-protocol` (252.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket` -> `mc-protocol` (252.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundPlayerInputPacket` -> `mc-protocol` (252.3s)
- [FAIL] `net.minecraft.nbt.EndTag` -> `mc-nbt` (1024.3s)
- [PASS] `net.minecraft.nbt.NbtAccounter` -> `mc-nbt` (1024.3s)
- [PASS] `net.minecraft.nbt.StreamTagVisitor` -> `mc-nbt` (1024.3s)
- [PASS] `net.minecraft.nbt.NumericTag` -> `mc-nbt` (1024.3s)
- [PASS] `net.minecraft.nbt.StringTagVisitor` -> `mc-nbt` (775.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.NbtProviders` -> `mc-nbt` (160.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.StorageNbtProvider` -> `mc-nbt` (160.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.NbtProvider` -> `mc-nbt` (160.7s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.nbt.LootNbtProviderType` -> `mc-nbt` (160.7s)
- [FAIL] `net.minecraft.network.protocol.ProtocolCodecBuilder` -> `mc-protocol` (702.8s)
- [FAIL] `net.minecraft.network.protocol.Packet` -> `mc-protocol` (702.8s)
