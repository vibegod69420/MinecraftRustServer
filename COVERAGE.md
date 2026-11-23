# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1585 / 7163 (22.128%)
- **Mapped Methods**: 7457 / 66676 (11.184%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2067 | Verified / Active |
| `mc-nbt` | 1720 | Verified / Active |
| `mc-protocol` | 3394 | Verified / Active |
| `mc-worldgen` | 2120 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
