# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1259 / 7163 (17.576%)
- **Mapped Methods**: 5388 / 66676 (8.081%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1901 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 1995 | Verified / Active |
| `mc-worldgen` | 1323 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
