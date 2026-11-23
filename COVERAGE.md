# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 883 / 7163 (12.327%)
- **Mapped Methods**: 3510 / 66676 (5.264%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1257 | Verified / Active |
| `mc-nbt` | 1130 | Verified / Active |
| `mc-protocol` | 993 | Verified / Active |
| `mc-worldgen` | 872 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
