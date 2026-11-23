# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1020 / 7163 (14.240%)
- **Mapped Methods**: 4178 / 66676 (6.266%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1486 | Verified / Active |
| `mc-nbt` | 1200 | Verified / Active |
| `mc-protocol` | 1369 | Verified / Active |
| `mc-worldgen` | 1118 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
