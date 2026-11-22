# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:04:52 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 319 / 66676 (0.478%)
- **Mapped Java Classes**: 99 / 7163 (1.382%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 263 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 36 mapped methods
- `mc-worldgen`: 55 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.valueproviders.ConstantFloat` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.ConstantInt` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.MultipliedFloats` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.SampledFloat` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.IntProviderType` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.FloatProviderType` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.nbt.CollectionTag` -> `mc-nbt` (275.2s)
- [PASS] `net.minecraft.nbt.TagVisitor` -> `mc-nbt` (275.2s)
- [PASS] `net.minecraft.nbt.TagTypes` -> `mc-nbt` (275.2s)
- [PASS] `net.minecraft.nbt.ReportedNbtException` -> `mc-nbt` (275.2s)
- [PASS] `net.minecraft.nbt.PrimitiveTag` -> `mc-nbt` (275.2s)
- [PASS] `net.minecraft.nbt.NbtAccounterException` -> `mc-nbt` (275.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.AlwaysTrueTest` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.PosRuleTest` -> `mc-worldgen` (273.2s)
- [PASS] `net.minecraft.world.level.levelgen.structure.templatesystem.LiquidSettings` -> `mc-worldgen` (273.2s)
