# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 658 / 7163 (9.186%)
- **Mapped Methods**: 2595 / 66676 (3.892%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 963 | Verified / Active |
| `mc-nbt` | 796 | Verified / Active |
| `mc-protocol` | 644 | Verified / Active |
| `mc-worldgen` | 685 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
