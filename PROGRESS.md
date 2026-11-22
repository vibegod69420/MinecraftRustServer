# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:05:50 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 355 / 66676 (0.532%)
- **Mapped Java Classes**: 116 / 7163 (1.619%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 288 mapped methods
- `mc-nbt`: 60 mapped methods
- `mc-protocol`: 59 mapped methods
- `mc-worldgen`: 58 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.network.protocol.login.custom.DiscardedQueryPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.network.protocol.login.custom.CustomQueryPayload` -> `mc-protocol` (70.1s)
- [PASS] `net.minecraft.util.random.WeightedList` -> `mc-math` (314.8s)
- [PASS] `net.minecraft.network.LocalFrameEncoder` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.DisconnectionDetails` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.LocalFrameDecoder` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.ConnectionProtocol` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.ServerboundPacketListener` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.network.ClientboundPacketListener` -> `mc-protocol` (103.9s)
- [PASS] `net.minecraft.util.valueproviders.ConstantFloat` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.ConstantInt` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.MultipliedFloats` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.SampledFloat` -> `mc-math` (297.6s)
- [PASS] `net.minecraft.util.valueproviders.IntProviderType` -> `mc-math` (297.6s)
