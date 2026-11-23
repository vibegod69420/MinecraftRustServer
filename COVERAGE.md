# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1264 / 7163 (17.646%)
- **Mapped Methods**: 5419 / 66676 (8.127%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1901 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 2030 | Verified / Active |
| `mc-worldgen` | 1338 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
