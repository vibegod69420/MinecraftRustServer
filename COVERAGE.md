# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 861 / 7163 (12.020%)
- **Mapped Methods**: 3425 / 66676 (5.137%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1235 | Verified / Active |
| `mc-nbt` | 1113 | Verified / Active |
| `mc-protocol` | 952 | Verified / Active |
| `mc-worldgen` | 858 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
