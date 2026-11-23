# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1289 / 7163 (17.995%)
- **Mapped Methods**: 5637 / 66676 (8.454%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2009 | Verified / Active |
| `mc-nbt` | 1541 | Verified / Active |
| `mc-protocol` | 2112 | Verified / Active |
| `mc-worldgen` | 1380 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
