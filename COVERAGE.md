# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 366 / 7163 (5.110%)
- **Mapped Methods**: 1224 / 66676 (1.836%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 565 | Verified / Active |
| `mc-nbt` | 304 | Verified / Active |
| `mc-protocol` | 273 | Verified / Active |
| `mc-worldgen` | 334 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
