# Progress Log (`PROGRESS.md`)

- **Updated**: 2026-11-23 09:48:29 UTC
- **Phase**: Phase A (Faithful 1:1 Port)
- **Mapped Java Methods**: 8717 / 66676 (13.074%)
- **Mapped Java Classes**: 1968 / 7163 (27.474%)
- **Oracle Step 0**: Deterministic self-test PASSING (world Merkle + RNG trace bit-identical)

## Crates
- `mc-data`: 6 mapped methods
- `mc-math`: 3292 mapped methods
- `mc-nbt`: 1720 mapped methods
- `mc-protocol`: 3408 mapped methods
- `mc-worldgen`: 2648 mapped methods

## Recent Verified Work
- [PASS] `net.minecraft.util.eventlog.JsonEventLog` -> `mc-math` (1622.9s)
- [PASS] `net.minecraft.util.eventlog.JsonEventLogReader` -> `mc-math` (1622.9s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemCustomNameToComponentFix` -> `mc-math` (1622.9s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsGraphicsModeSplitFix` -> `mc-math` (1622.9s)
- [PASS] `net.minecraft.util.datafix.fixes.AddNewChoices` -> `mc-math` (1622.9s)
- [PASS] `net.minecraft.util.datafix.fixes.BlockEntityJukeboxFix` -> `mc-math` (1622.9s)
- [PASS] `net.minecraft.util.datafix.fixes.IglooMetadataRemovalFix` -> `mc-math` (1622.8s)
- [PASS] `net.minecraft.util.datafix.fixes.WorldSpawnDataFix` -> `mc-math` (1622.8s)
- [PASS] `net.minecraft.util.datafix.fixes.ItemRenameFix` -> `mc-math` (1622.8s)
- [PASS] `net.minecraft.util.datafix.fixes.OptionsKeyTranslationFix` -> `mc-math` (1622.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix` -> `mc-math` (1622.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.ChunkTicketUnpackPosFix` -> `mc-math` (1622.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.AbstractBlockPropertyFix` -> `mc-math` (1622.8s)
- [FAIL] `net.minecraft.util.datafix.fixes.EntityHorseSplitFix` -> `mc-math` (1622.8s)
- [PASS] `net.minecraft.util.datafix.schemas.V4292` -> `mc-math` (689.6s)
