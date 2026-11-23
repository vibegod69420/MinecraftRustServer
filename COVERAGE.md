# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1119 / 7163 (15.622%)
- **Mapped Methods**: 4683 / 66676 (7.024%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1651 | Verified / Active |
| `mc-nbt` | 1390 | Verified / Active |
| `mc-protocol` | 1518 | Verified / Active |
| `mc-worldgen` | 1260 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
