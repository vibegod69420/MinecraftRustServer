# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:28:50 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1134 / 66676 (1.701%)
- **Mapped Java Classes**: 342 / 7163 (4.774%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 565 mapped methods
- `mc-nbt`: 244 mapped methods
- `mc-protocol`: 236 mapped methods
- `mc-worldgen`: 331 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.IntArrayTag` -> `mc-nbt` (247.5s)
- [FAIL] `net.minecraft.core.component.TypedDataComponent` -> `mc-math` (549.9s)
- [FAIL] `net.minecraft.core.component.DataComponentGetter` -> `mc-math` (549.9s)
- [PASS] `net.minecraft.core.component.DataComponentHolder` -> `mc-math` (549.9s)
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
