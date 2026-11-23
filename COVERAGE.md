# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1276 / 7163 (17.814%)
- **Mapped Methods**: 5487 / 66676 (8.229%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1939 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 2053 | Verified / Active |
| `mc-worldgen` | 1355 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
