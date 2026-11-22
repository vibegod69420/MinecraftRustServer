# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 23:47:50 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2883 / 66676 (4.324%)
- **Mapped Java Classes**: 727 / 7163 (10.149%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 1075 mapped methods
- `mc-nbt`: 954 mapped methods
- `mc-protocol`: 701 mapped methods
- `mc-worldgen`: 701 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.LinearCongruentialGenerator` -> `mc-math` (279.3s)
- [PASS] `net.minecraft.util.BitStorage` -> `mc-math` (279.3s)
- [PASS] `net.minecraft.util.SignatureUpdater` -> `mc-math` (279.3s)
- [PASS] `net.minecraft.util.ProgressListener` -> `mc-math` (279.3s)
- [FAIL] `net.minecraft.world.level.storage.loot.predicates.LootItemCondition` -> `mc-nbt` (667.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.DamageSourceCondition` -> `mc-nbt` (667.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.ValueCheckCondition` -> `mc-nbt` (667.5s)
- [PASS] `net.minecraft.world.level.storage.loot.predicates.AllOfCondition` -> `mc-nbt` (667.5s)
- [PASS] `net.minecraft.util.Unit` -> `mc-math` (134.1s)
- [PASS] `net.minecraft.util.TimeSource` -> `mc-math` (134.1s)
- [PASS] `net.minecraft.util.Tuple` -> `mc-math` (134.1s)
- [PASS] `net.minecraft.util.ColorRGBA` -> `mc-math` (134.1s)
- [PASS] `net.minecraft.core.component.DataComponents` -> `mc-math` (290.7s)
- [PASS] `net.minecraft.network.chat.contents.objects.AtlasSprite` -> `mc-protocol` (111.3s)
- [PASS] `net.minecraft.network.chat.contents.objects.PlayerSprite` -> `mc-protocol` (111.3s)
