# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1198 / 7163 (16.725%)
- **Mapped Methods**: 5116 / 66676 (7.673%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1828 | Verified / Active |
| `mc-nbt` | 1524 | Verified / Active |
| `mc-protocol` | 1740 | Verified / Active |
| `mc-worldgen` | 1283 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
