# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 990 / 7163 (13.821%)
- **Mapped Methods**: 4023 / 66676 (6.034%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1430 | Verified / Active |
| `mc-nbt` | 1173 | Verified / Active |
| `mc-protocol` | 1324 | Verified / Active |
| `mc-worldgen` | 1042 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
