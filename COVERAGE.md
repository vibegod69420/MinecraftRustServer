# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 2106 / 7163 (29.401%)
- **Mapped Methods**: 9171 / 66676 (13.755%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 3909 | Verified / Active |
| `mc-nbt` | 1720 | Verified / Active |
| `mc-protocol` | 3408 | Verified / Active |
| `mc-worldgen` | 2648 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
