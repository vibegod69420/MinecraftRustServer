# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:56:22 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2036 / 66676 (3.054%)
- **Mapped Java Classes**: 559 / 7163 (7.804%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 778 mapped methods
- `mc-nbt`: 614 mapped methods
- `mc-protocol`: 490 mapped methods
- `mc-worldgen`: 540 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks` -> `mc-protocol` (173.7s)
- [FAIL] `net.minecraft.nbt.ListTag` -> `mc-nbt` (1615.8s)
- [PASS] `net.minecraft.core.component.DataComponentType` -> `mc-math` (398.7s)
- [PASS] `net.minecraft.core.component.DataComponentExactPredicate` -> `mc-math` (273.6s)
- [PASS] `net.minecraft.network.protocol.login.ServerboundHelloPacket` -> `mc-protocol` (189.3s)
- [PASS] `net.minecraft.core.dispenser.MinecartDispenseItemBehavior` -> `mc-math` (565.0s)
- [PASS] `net.minecraft.core.dispenser.DefaultDispenseItemBehavior` -> `mc-math` (565.0s)
- [FAIL] `net.minecraft.nbt.TextComponentTagVisitor` -> `mc-nbt` (1549.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundCooldownPacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundSetTimePacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.network.protocol.game.ClientboundRecipeBookRemovePacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket` -> `mc-protocol` (238.4s)
- [PASS] `net.minecraft.core.RegistryAccess` -> `mc-math` (258.4s)
- [FAIL] `net.minecraft.world.level.storage.ValueInput` -> `mc-nbt` (521.5s)
- [FAIL] `net.minecraft.world.level.storage.LevelResource` -> `mc-nbt` (521.5s)
