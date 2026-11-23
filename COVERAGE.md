# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1147 / 7163 (16.013%)
- **Mapped Methods**: 4865 / 66676 (7.296%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1731 | Verified / Active |
| `mc-nbt` | 1437 | Verified / Active |
| `mc-protocol` | 1624 | Verified / Active |
| `mc-worldgen` | 1283 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
