# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-24 04:04:25 UTC
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
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkProtoTickListFix` -> `mc-math` (828.7s)
- [FAIL] `net.minecraft.util.datafix.fixes.MemoryExpiryDataFix` -> `mc-math` (575.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix` -> `mc-math` (575.1s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkHeightAndBiomeFix` -> `mc-math` (2359.9s)
- [FAIL] `net.minecraft.world.level.chunk.PalettedContainer` -> `mc-worldgen` (3493.1s)
- [FAIL] `net.minecraft.world.level.chunk.storage.IOWorker` -> `mc-worldgen` (3324.8s)
- [FAIL] `net.minecraft.util.profiling.jfr.serialize.JfrResultJsonSerializer` -> `mc-math` (1497.2s)
- [FAIL] `net.minecraft.world.level.chunk.ChunkAccess` -> `mc-worldgen` (2528.4s)
- [FAIL] `net.minecraft.util.datafix.fixes.ItemStackComponentizationFix` -> `mc-math` (581.9s)
- [FAIL] `net.minecraft.util.profiling.jfr.JfrProfiler` -> `mc-math` (1111.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkPalettedStorageFix` -> `mc-math` (447.6s)
- [FAIL] `net.minecraft.util.datafix.fixes.TrialSpawnerConfigInRegistryFix` -> `mc-math` (833.2s)
- [FAIL] `net.minecraft.util.datafix.schemas.V1460` -> `mc-math` (440.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityBlockStateFix` -> `mc-math` (502.0s)
- [FAIL] `net.minecraft.util.datafix.fixes.GameRuleRegistryFix` -> `mc-math` (989.2s)
