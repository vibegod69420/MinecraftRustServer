# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1062 / 7163 (14.826%)
- **Mapped Methods**: 4344 / 66676 (6.515%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1533 | Verified / Active |
| `mc-nbt` | 1236 | Verified / Active |
| `mc-protocol` | 1479 | Verified / Active |
| `mc-worldgen` | 1150 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
