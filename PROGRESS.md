# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 09:10:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 9539 / 66676 (14.306%)
- **Mapped Java Classes**: 2164 / 7163 (30.211%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 4163 mapped methods
- `mc-nbt`: 1837 mapped methods
- `mc-protocol`: 3493 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.world.level.storage.WorldData` -> `mc-nbt` (248.3s)
- [FAIL] `net.minecraft.world.level.storage.ServerLevelData` -> `mc-nbt` (248.3s)
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (248.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.CompositeEntryBase` -> `mc-nbt` (377.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.EntryGroup` -> `mc-nbt` (377.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.entries.SequentialEntry` -> `mc-nbt` (377.6s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (92.8s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (98.5s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (64.5s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (57.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkHeightAndBiomeFix` -> `mc-math` (3209.5s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkGeneratorStructureState` -> `mc-worldgen` (2871.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (1500.7s)
- [FAIL] `net.minecraft.world.level.chunk.storage.RegionFile` -> `mc-worldgen` (1996.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (978.0s)
