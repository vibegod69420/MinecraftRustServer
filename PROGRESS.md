# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:24:51 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 960 / 66676 (1.440%)
- **Mapped Java Classes**: 303 / 7163 (4.230%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 532 mapped methods
- `mc-nbt`: 145 mapped methods
- `mc-protocol`: 208 mapped methods
- `mc-worldgen`: 305 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.nbt.visitors.CollectFields` -> `mc-nbt` (347.0s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.ColumnFeatureConfiguration` -> `mc-worldgen` (140.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.LayerConfiguration` -> `mc-worldgen` (140.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.SimpleBlockConfiguration` -> `mc-worldgen` (140.2s)
- [PASS] `net.minecraft.world.level.levelgen.feature.configurations.TwistingVinesConfig` -> `mc-worldgen` (140.2s)
- [PASS] `net.minecraft.world.level.levelgen.synth.BlendedNoise` -> `mc-math` (341.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.WeightedStateProvider` -> `mc-worldgen` (161.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.RotatedBlockProvider` -> `mc-worldgen` (161.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.BlockStateProvider` -> `mc-worldgen` (161.1s)
- [PASS] `net.minecraft.world.level.levelgen.feature.stateproviders.SimpleStateProvider` -> `mc-worldgen` (161.1s)
- [FAIL] `net.minecraft.world.level.levelgen.structure.placement.StructurePlacementType` -> `mc-worldgen` (307.5s)
- [PASS] `net.minecraft.world.level.levelgen.structure.placement.RandomSpreadType` -> `mc-worldgen` (307.5s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundServerLinksPacket` -> `mc-protocol` (230.8s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundDisconnectPacket` -> `mc-protocol` (230.8s)
- [PASS] `net.minecraft.network.protocol.common.ClientboundClearDialogPacket` -> `mc-protocol` (230.8s)
