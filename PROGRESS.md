# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-22 22:45:25 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 1673 / 66676 (2.509%)
- **Mapped Java Classes**: 486 / 7163 (6.785%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 679 mapped methods
- `mc-nbt`: 494 mapped methods
- `mc-protocol`: 329 mapped methods
- `mc-worldgen`: 494 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.network.PacketSendListener` -> `mc-protocol` (143.4s)
- [PASS] `net.minecraft.network.RateKickingConnection` -> `mc-protocol` (143.4s)
- [PASS] `net.minecraft.network.ProtocolInfo` -> `mc-protocol` (143.4s)
- [PASS] `net.minecraft.network.HandlerNames` -> `mc-protocol` (143.4s)
- [FAIL] `net.minecraft.world.level.levelgen.GenerationStep` -> `mc-worldgen` (274.5s)
- [PASS] `net.minecraft.world.level.levelgen.ThreadSafeLegacyRandomSource` -> `mc-worldgen` (274.5s)
- [PASS] `net.minecraft.world.level.levelgen.SingleThreadedRandomSource` -> `mc-worldgen` (274.5s)
- [PASS] `net.minecraft.world.level.levelgen.feature.SimpleRandomSelectorFeature` -> `mc-worldgen` (808.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.RandomSelectorFeature` -> `mc-worldgen` (808.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.RandomBooleanSelectorFeature` -> `mc-worldgen` (808.6s)
- [PASS] `net.minecraft.world.level.levelgen.feature.ChorusPlantFeature` -> `mc-worldgen` (808.6s)
- [FAIL] `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProviders` -> `mc-nbt` (398.1s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.FixedScoreboardNameProvider` -> `mc-nbt` (398.1s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.ScoreboardNameProvider` -> `mc-nbt` (398.1s)
- [PASS] `net.minecraft.world.level.storage.loot.providers.score.LootScoreProviderType` -> `mc-nbt` (398.1s)
