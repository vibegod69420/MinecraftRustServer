# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 316 / 7163 (4.412%)
- **Mapped Methods**: 1017 / 66676 (1.525%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 532 | Verified / Active |
| `mc-nbt` | 193 | Verified / Active |
| `mc-protocol` | 213 | Verified / Active |
| `mc-worldgen` | 315 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
