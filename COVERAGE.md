# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1097 / 7163 (15.315%)
- **Mapped Methods**: 4522 / 66676 (6.782%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1577 | Verified / Active |
| `mc-nbt` | 1298 | Verified / Active |
| `mc-protocol` | 1518 | Verified / Active |
| `mc-worldgen` | 1233 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
