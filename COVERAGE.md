# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 2139 / 7163 (29.862%)
- **Mapped Methods**: 9377 / 66676 (14.063%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 4090 | Verified / Active |
| `mc-nbt` | 1736 | Verified / Active |
| `mc-protocol` | 3450 | Verified / Active |
| `mc-worldgen` | 2648 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
