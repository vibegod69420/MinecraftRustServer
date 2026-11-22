# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:27:40 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1086 / 66676 (1.629%)
- **Mapped Java Classes**: 336 / 7163 (4.691%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 548 mapped methods
- `mc-nbt`: 218 mapped methods
- `mc-protocol`: 236 mapped methods
- `mc-worldgen`: 324 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.particles.ExplosionParticleInfo` -> `mc-math` (211.3s)
- [PASS] `net.minecraft.core.particles.ItemParticleOption` -> `mc-math` (211.3s)
- [PASS] `net.minecraft.core.particles.TrailParticleOption` -> `mc-math` (211.3s)
- [PASS] `net.minecraft.core.particles.PowerParticleOption` -> `mc-math` (211.3s)
- [PASS] `net.minecraft.nbt.ByteArrayTag` -> `mc-nbt` (240.0s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.PieceGenerator` -> `mc-worldgen` (431.7s)
- [PASS] `net.minecraft.world.level.levelgen.structure.pieces.StructurePieceSerializationContext` -> `mc-worldgen` (431.7s)
- [PASS] `net.minecraft.network.Varint21LengthFieldPrepender` -> `mc-protocol` (428.7s)
- [PASS] `net.minecraft.network.ProtocolSwapHandler` -> `mc-protocol` (428.7s)
- [PASS] `net.minecraft.network.PacketBundleUnpacker` -> `mc-protocol` (428.7s)
- [PASS] `net.minecraft.network.MonitoredLocalFrameDecoder` -> `mc-protocol` (428.7s)
- [PASS] `net.minecraft.nbt.TagParser` -> `mc-nbt` (225.3s)
- [PASS] `net.minecraft.nbt.ByteTag` -> `mc-nbt` (225.3s)
- [FAIL] `net.minecraft.nbt.LongTag` -> `mc-nbt` (397.9s)
- [FAIL] `net.minecraft.nbt.ShortTag` -> `mc-nbt` (397.9s)
