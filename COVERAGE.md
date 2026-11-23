# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1045 / 7163 (14.589%)
- **Mapped Methods**: 4283 / 66676 (6.424%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1533 | Verified / Active |
| `mc-nbt` | 1227 | Verified / Active |
| `mc-protocol` | 1399 | Verified / Active |
| `mc-worldgen` | 1150 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
