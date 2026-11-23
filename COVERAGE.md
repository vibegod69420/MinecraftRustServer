# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1243 / 7163 (17.353%)
- **Mapped Methods**: 5310 / 66676 (7.964%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1873 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 1955 | Verified / Active |
| `mc-worldgen` | 1298 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
