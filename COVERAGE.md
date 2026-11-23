# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1910 / 7163 (26.665%)
- **Mapped Methods**: 8558 / 66676 (12.835%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 3031 | Verified / Active |
| `mc-nbt` | 1720 | Verified / Active |
| `mc-protocol` | 3408 | Verified / Active |
| `mc-worldgen` | 2648 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
