# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1234 / 7163 (17.227%)
- **Mapped Methods**: 5271 / 66676 (7.905%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1868 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 1926 | Verified / Active |
| `mc-worldgen` | 1283 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
