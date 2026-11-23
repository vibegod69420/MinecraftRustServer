# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 893 / 7163 (12.467%)
- **Mapped Methods**: 3563 / 66676 (5.344%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1257 | Verified / Active |
| `mc-nbt` | 1130 | Verified / Active |
| `mc-protocol` | 1068 | Verified / Active |
| `mc-worldgen` | 872 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
