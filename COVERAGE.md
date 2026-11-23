# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1071 / 7163 (14.952%)
- **Mapped Methods**: 4383 / 66676 (6.574%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1533 | Verified / Active |
| `mc-nbt` | 1236 | Verified / Active |
| `mc-protocol` | 1483 | Verified / Active |
| `mc-worldgen` | 1202 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
