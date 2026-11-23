# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1085 / 7163 (15.147%)
- **Mapped Methods**: 4469 / 66676 (6.703%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1563 | Verified / Active |
| `mc-nbt` | 1289 | Verified / Active |
| `mc-protocol` | 1490 | Verified / Active |
| `mc-worldgen` | 1214 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
