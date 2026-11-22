# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:21:30 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 733 / 66676 (1.099%)
- **Mapped Java Classes**: 245 / 7163 (3.420%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 435 mapped methods
- `mc-nbt`: 96 mapped methods
- `mc-protocol`: 155 mapped methods
- `mc-worldgen`: 245 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacement` -> `mc-worldgen` (149.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.RootPlacerType` -> `mc-worldgen` (149.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.rootplacers.AboveRootPlacement` -> `mc-worldgen` (149.5s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.ReplaceablePredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.AllOfPredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.AnyOfPredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.CombiningPredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.SolidPredicate` -> `mc-worldgen` (478.3s)
- [PASS] `net.minecraft.world.level.levelgen.blockpredicates.TrueBlockPredicate` -> `mc-worldgen` (478.3s)
- [FAIL] `net.minecraft.nbt.TagParser` -> `mc-nbt` (281.5s)
- [FAIL] `net.minecraft.nbt.ByteTag` -> `mc-nbt` (281.5s)
- [FAIL] `net.minecraft.nbt.LongTag` -> `mc-nbt` (281.5s)
- [PASS] `net.minecraft.core.HolderOwner` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.RegistryCodecs` -> `mc-math` (327.7s)
- [PASS] `net.minecraft.core.QuartPos` -> `mc-math` (327.7s)
