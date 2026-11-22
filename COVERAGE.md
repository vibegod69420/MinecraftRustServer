# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 492 / 7163 (6.869%)
- **Mapped Methods**: 1698 / 66676 (2.547%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 679 | Verified / Active |
| `mc-nbt` | 496 | Verified / Active |
| `mc-protocol` | 357 | Verified / Active |
| `mc-worldgen` | 494 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
