# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:07:00 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 388 / 66676 (0.582%)
- **Mapped Java Classes**: 127 / 7163 (1.773%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 330 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 71 mapped methods
- `mc-worldgen`: 60 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.nbt.visitors.CollectFields` -> `mc-nbt` (426.0s)
- [FAIL] `net.minecraft.nbt.visitors.SkipAll` -> `mc-nbt` (426.0s)
- [FAIL] `net.minecraft.nbt.visitors.SkipFields` -> `mc-nbt` (426.0s)
- [FAIL] `net.minecraft.nbt.visitors.FieldTree` -> `mc-nbt` (426.0s)
- [FAIL] `net.minecraft.nbt.visitors.FieldSelector` -> `mc-nbt` (426.0s)
- [PASS] `net.minecraft.network.syncher.EntityDataSerializer` -> `mc-protocol` (256.7s)
- [PASS] `net.minecraft.network.syncher.EntityDataAccessor` -> `mc-protocol` (256.7s)
- [PASS] `net.minecraft.network.syncher.SyncedDataHolder` -> `mc-protocol` (256.7s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (364.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.PostPlacementProcessor` -> `mc-worldgen` (364.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.TerrainAdjustment` -> `mc-worldgen` (364.2s)
- [FAIL] `net.minecraft.network.chat.numbers.NumberFormat` -> `mc-protocol` (356.5s)
- [PASS] `net.minecraft.network.chat.numbers.NumberFormatType` -> `mc-protocol` (356.5s)
- [PASS] `net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.network.protocol.login.custom.DiscardedQueryPayload` -> `mc-protocol` (70.1s)
