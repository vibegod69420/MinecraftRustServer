# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 553 / 7163 (7.720%)
- **Mapped Methods**: 2001 / 66676 (3.001%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 748 | Verified / Active |
| `mc-nbt` | 614 | Verified / Active |
| `mc-protocol` | 480 | Verified / Active |
| `mc-worldgen` | 540 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
