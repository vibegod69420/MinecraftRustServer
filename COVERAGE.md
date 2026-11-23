# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 800 / 7163 (11.168%)
- **Mapped Methods**: 3233 / 66676 (4.849%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1178 | Verified / Active |
| `mc-nbt` | 1092 | Verified / Active |
| `mc-protocol` | 876 | Verified / Active |
| `mc-worldgen` | 768 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
