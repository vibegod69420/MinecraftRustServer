# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 900 / 7163 (12.565%)
- **Mapped Methods**: 3592 / 66676 (5.387%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1257 | Verified / Active |
| `mc-nbt` | 1147 | Verified / Active |
| `mc-protocol` | 1068 | Verified / Active |
| `mc-worldgen` | 895 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
