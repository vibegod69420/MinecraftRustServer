# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 21:10:26 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9417 / 66676 (14.123%)
- **Mapped Java Classes**: 2142 / 7163 (29.904%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4092 mapped methods
- `mc-nbt`: 1769 mapped methods
- `mc-protocol`: 3460 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetObjectivePacket` -> `mc-protocol` (1041.0s)
- [FAIL] `net.minecraft.core.particles.DustColorTransitionOptions` -> `mc-math` (499.9s)
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (391.0s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (391.0s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (391.0s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetBookCoverFunction` -> `mc-nbt` (381.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetFireworksFunction` -> `mc-nbt` (381.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.functions.SetOminousBottleAmplifierFunction` -> `mc-nbt` (381.1s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (320.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (320.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (320.3s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (122.4s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (62.1s)
- [PASS] `net.minecraft.world.level.storage.ValueInputContextHelper` -> `mc-nbt` (2759.8s)
- [FAIL] `net.minecraft.util.Crypt` -> `mc-math` (11233.4s)
