# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 582 / 7163 (8.125%)
- **Mapped Methods**: 2165 / 66676 (3.247%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 886 | Verified / Active |
| `mc-nbt` | 614 | Verified / Active |
| `mc-protocol` | 528 | Verified / Active |
| `mc-worldgen` | 565 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
