# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:39:33 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1458 / 66676 (2.187%)
- **Mapped Java Classes**: 428 / 7163 (5.975%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 591 mapped methods
- `mc-nbt`: 434 mapped methods
- `mc-protocol`: 309 mapped methods
- `mc-worldgen`: 421 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.core.particles.ColorParticleOption` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.core.particles.BlockParticleOption` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.core.particles.DustParticleOptions` -> `mc-math` (638.5s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootPoolEntry` -> `mc-nbt` (313.8s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.EmptyLootItem` -> `mc-nbt` (313.8s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.ComposableEntryContainer` -> `mc-nbt` (313.8s)
- [PASS] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryType` -> `mc-nbt` (313.8s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.ConstantHeight` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.HeightProviderType` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.WeightedListHeight` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.world.level.levelgen.heightproviders.HeightProvider` -> `mc-worldgen` (267.6s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket` -> `mc-protocol` (252.3s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket` -> `mc-protocol` (252.3s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundPlayerCombatEnterPacket` -> `mc-protocol` (252.3s)
