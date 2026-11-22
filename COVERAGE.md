# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 549 / 7163 (7.664%)
- **Mapped Methods**: 1980 / 66676 (2.970%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 744 | Verified / Active |
| `mc-nbt` | 594 | Verified / Active |
| `mc-protocol` | 480 | Verified / Active |
| `mc-worldgen` | 540 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
