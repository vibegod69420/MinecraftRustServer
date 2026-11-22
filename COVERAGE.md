# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 565 / 7163 (7.888%)
- **Mapped Methods**: 2050 / 66676 (3.075%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 778 | Verified / Active |
| `mc-nbt` | 614 | Verified / Active |
| `mc-protocol` | 490 | Verified / Active |
| `mc-worldgen` | 558 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
