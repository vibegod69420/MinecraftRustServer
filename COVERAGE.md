# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1003 / 7163 (14.002%)
- **Mapped Methods**: 4104 / 66676 (6.155%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1478 | Verified / Active |
| `mc-nbt` | 1191 | Verified / Active |
| `mc-protocol` | 1333 | Verified / Active |
| `mc-worldgen` | 1064 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
