# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:58:59 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2077 / 66676 (3.115%)
- **Mapped Java Classes**: 570 / 7163 (7.958%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 814 mapped methods
- `mc-nbt`: 614 mapped methods
- `mc-protocol`: 490 mapped methods
- `mc-worldgen`: 558 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.core.dispenser.ShearsDispenseItemBehavior` -> `mc-math` (1392.7s)
- [PASS] `net.minecraft.core.dispenser.BoatDispenseItemBehavior` -> `mc-math` (1392.7s)
- [PASS] `net.minecraft.core.dispenser.ProjectileDispenseBehavior` -> `mc-math` (1392.7s)
- [PASS] `net.minecraft.world.level.levelgen.feature.WeightedPlacedFeature` -> `mc-worldgen` (686.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.EndIslandFeature` -> `mc-worldgen` (686.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.RandomPatchFeature` -> `mc-worldgen` (686.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.VinesFeature` -> `mc-worldgen` (686.2s)
- [PASS] `net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks` -> `mc-protocol` (173.7s)
- [FAIL] `net.minecraft.nbt.ListTag` -> `mc-nbt` (1615.8s)
- [PASS] `net.minecraft.core.component.DataComponentType` -> `mc-math` (398.7s)
- [PASS] `net.minecraft.core.component.DataComponentExactPredicate` -> `mc-math` (273.6s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundHelloPacket` -> `mc-protocol` (189.3s)
- [PASS] `net.minecraft.core.dispenser.MinecartDispenseItemBehavior` -> `mc-math` (565.0s)
- [PASS] `net.minecraft.core.dispenser.DefaultDispenseItemBehavior` -> `mc-math` (565.0s)
- [FAIL] `net.minecraft.nbt.TextComponentTagVisitor` -> `mc-nbt` (1549.4s)
