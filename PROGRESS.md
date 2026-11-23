# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 00:08:06 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 3290 / 66676 (4.934%)
- **Mapped Java Classes**: 818 / 7163 (11.420%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1182 mapped methods
- `mc-nbt`: 1098 mapped methods
- `mc-protocol`: 882 mapped methods
- `mc-worldgen`: 818 mapped methods

## Recent Verified Work
- [FAIL] `net.minecraft.nbt.TextComponentTagVisitor` -> `mc-nbt` (2314.4s)
- [PASS] `net.minecraft.util.debug.DebugBreezeInfo` -> `mc-math` (1257.3s)
- [PASS] `net.minecraft.util.debug.DebugPathInfo` -> `mc-math` (1257.3s)
- [PASS] `net.minecraft.util.debug.DebugGameEventListenerInfo` -> `mc-math` (1257.3s)
- [PASS] `net.minecraft.util.debug.DebugValueSource` -> `mc-math` (1257.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetItemCountFunction` -> `mc-nbt` (811.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.ModifyContainerContents` -> `mc-nbt` (811.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetWrittenBookPagesFunction` -> `mc-nbt` (811.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetInstrumentFunction` -> `mc-nbt` (811.3s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SmeltItemFunction` -> `mc-nbt` (979.4s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.FillPlayerHead` -> `mc-nbt` (979.4s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SequenceFunction` -> `mc-nbt` (979.4s)
- [PASS] `net.minecraft.world.level.storage.loot.functions.SetWritableBookPagesFunction` -> `mc-nbt` (979.4s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.SimpleBlockFeature` -> `mc-worldgen` (2190.6s)
- [FAIL] `net.minecraft.world.level.levelgen.feature.CoralTreeFeature` -> `mc-worldgen` (2190.6s)
