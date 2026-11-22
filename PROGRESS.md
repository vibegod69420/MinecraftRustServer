# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:18:58 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 712 / 66676 (1.068%)
- **Mapped Java Classes**: 239 / 7163 (3.337%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 435 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 155 mapped methods
- `mc-worldgen`: 223 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.nbt.TagParser` -> `mc-nbt` (281.5s)
- [FAIL] `net.minecraft.nbt.ByteTag` -> `mc-nbt` (281.5s)
- [FAIL] `net.minecraft.nbt.LongTag` -> `mc-nbt` (281.5s)
- [PASS] `net.minecraft.core.HolderOwner` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.RegistryCodecs` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.QuartPos` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.DefaultedRegistry` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.WritableRegistry` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.RegistrationInfo` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.particles.ShriekParticleOption` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.SculkChargeParticleOptions` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.SimpleParticleType` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.ScalableParticleOptionsBase` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.ParticleType` -> `mc-math` (198.9s)
- [PASS] `net.minecraft.core.particles.ParticleLimit` -> `mc-math` (198.9s)
