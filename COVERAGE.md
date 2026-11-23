# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 921 / 7163 (12.858%)
- **Mapped Methods**: 3668 / 66676 (5.501%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1299 | Verified / Active |
| `mc-nbt` | 1157 | Verified / Active |
| `mc-protocol` | 1102 | Verified / Active |
| `mc-worldgen` | 910 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
