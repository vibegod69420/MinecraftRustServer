# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1109 / 7163 (15.482%)
- **Mapped Methods**: 4591 / 66676 (6.886%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1597 | Verified / Active |
| `mc-nbt` | 1347 | Verified / Active |
| `mc-protocol` | 1518 | Verified / Active |
| `mc-worldgen` | 1253 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
