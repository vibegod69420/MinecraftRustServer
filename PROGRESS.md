# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:44:46 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1587 / 66676 (2.380%)
- **Mapped Java Classes**: 464 / 7163 (6.478%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 636 mapped methods
- `mc-nbt`: 494 mapped methods
- `mc-protocol`: 309 mapped methods
- `mc-worldgen`: 459 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.particles.SpellParticleOption` -> `mc-math` (275.4s)
- [PASS] `net.minecraft.core.particles.VibrationParticleOption` -> `mc-math` (275.4s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.UnderwaterMagmaConfiguration` -> `mc-worldgen` (118.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.HugeMushroomFeatureConfiguration` -> `mc-worldgen` (118.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.ReplaceSphereConfiguration` -> `mc-worldgen` (118.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.DiskConfiguration` -> `mc-worldgen` (118.7s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootDataType` -> `mc-nbt` (167.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.ContainerComponentManipulator` -> `mc-nbt` (167.2s)
- [FAIL] `net.minecraft.world.level.storage.loot.LootContextUser` -> `mc-nbt` (167.2s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.LootNumberProviderType` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.ConstantValue` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.EnchantmentLevelProvider` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.number.NumberProvider` -> `mc-nbt` (357.6s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.LootItemFunction` -> `mc-nbt` (399.9s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.DiscardItem` -> `mc-nbt` (399.9s)
