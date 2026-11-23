# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1022 / 7163 (14.268%)
- **Mapped Methods**: 4188 / 66676 (6.281%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1491 | Verified / Active |
| `mc-nbt` | 1200 | Verified / Active |
| `mc-protocol` | 1369 | Verified / Active |
| `mc-worldgen` | 1123 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
