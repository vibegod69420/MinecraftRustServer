# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:38:08 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1418 / 66676 (2.127%)
- **Mapped Java Classes**: 412 / 7163 (5.752%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 591 mapped methods
- `mc-nbt`: 402 mapped methods
- `mc-protocol`: 309 mapped methods
- `mc-worldgen`: 408 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.ConstantHeight` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.HeightProviderType` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.HeightProvider` -> `mc-worldgen` (267.6s)
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
