# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:56:01 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 2031 / 66676 (3.046%)
- **Mapped Java Classes**: 558 / 7163 (7.790%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 778 mapped methods
- `mc-nbt`: 614 mapped methods
- `mc-protocol`: 484 mapped methods
- `mc-worldgen`: 540 mapped methods

## Recent Verified Work
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
- [FAIL] `net.minecraft.world.level.storage.ValueOutput` -> `mc-nbt` (521.5s)
