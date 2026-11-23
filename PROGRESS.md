# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 22:00:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9438 / 66676 (14.155%)
- **Mapped Java Classes**: 2145 / 7163 (29.946%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4092 mapped methods
- `mc-nbt`: 1775 mapped methods
- `mc-protocol`: 3478 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.loot.entries.AlternativesEntry` -> `mc-nbt` (315.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.LootPoolEntryContainer` -> `mc-nbt` (315.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.TagEntry` -> `mc-nbt` (315.2s)
- [PASS] `net.minecraft.world.level.storage.loot.LootDataType` -> `mc-nbt` (3274.1s)
- [FAIL] `net.minecraft.network.protocol.game.ClientboundCommandsPacket` -> `mc-protocol` (2893.0s)
- [FAIL] `net.minecraft.util.debug.TrackingDebugSynchronizer` -> `mc-math` (2953.8s)
- [FAIL] `net.minecraft.util.debug.LevelDebugSynchronizers` -> `mc-math` (3274.1s)
- [PASS] `net.minecraft.network.chat.contents.TranslatableContents` -> `mc-protocol` (3151.7s)
- [FAIL] `net.minecraft.core.RegistrySynchronization` -> `mc-math` (3274.1s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` -> `mc-protocol` (1041.0s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (499.9s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (391.0s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (391.0s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (391.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (381.1s)
