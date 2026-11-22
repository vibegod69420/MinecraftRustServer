# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:00:02 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 202 / 66676 (0.303%)
- **Mapped Java Classes**: 60 / 7163 (0.838%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 234 mapped methods
- `mc-nbt`: 8 mapped methods
- `mc-protocol`: 13 mapped methods
- `mc-worldgen`: 13 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.random.WeightedRandom` -> `mc-math` (737.8s)
- [PASS] `net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload` -> `mc-protocol` (157.3s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructurePieceAccessor` -> `mc-worldgen` (189.0s)
- [PASS] `net.minecraft.network.codec.StreamEncoder` -> `mc-protocol` (477.7s)
- [PASS] `net.minecraft.network.codec.StreamMemberEncoder` -> `mc-protocol` (141.8s)
- [FAIL] `net.minecraft.nbt.NbtAccounterException` -> `mc-nbt` (363.3s)
- [FAIL] `net.minecraft.network.codec.StreamDecoder` -> `mc-protocol` (425.6s)
- [PASS] `net.minecraft.nbt.NbtFormatException` -> `mc-nbt` (360.4s)
- [PASS] `net.minecraft.world.level.levelgen.Density` -> `mc-worldgen` (101.3s)
- [PASS] `net.minecraft.nbt.NbtException` -> `mc-nbt` (122.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.StructureCheckResult` -> `mc-worldgen` (103.0s)
- [PASS] `net.minecraft.util.random.Weighted` -> `mc-math` (43.4s)
- [PASS] `net.minecraft.util.profiling.metrics.profiling.MetricsRecorder` -> `mc-math` (124.0s)
- [PASS] `net.minecraft.util.RegistryContextSwapper` -> `mc-math` (162.7s)
- [PASS] `net.minecraft.util.profiling.metrics.MetricsSamplerProvider` -> `mc-math` (201.9s)
