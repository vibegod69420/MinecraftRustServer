# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1270 / 7163 (17.730%)
- **Mapped Methods**: 5458 / 66676 (8.186%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1924 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 2053 | Verified / Active |
| `mc-worldgen` | 1338 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
