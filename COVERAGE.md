# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 545 / 7163 (7.609%)
- **Mapped Methods**: 1961 / 66676 (2.941%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 744 | Verified / Active |
| `mc-nbt` | 594 | Verified / Active |
| `mc-protocol` | 461 | Verified / Active |
| `mc-worldgen` | 540 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
