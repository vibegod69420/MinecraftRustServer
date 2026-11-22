# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 342 / 7163 (4.774%)
- **Mapped Methods**: 1134 / 66676 (1.701%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 565 | Verified / Active |
| `mc-nbt` | 244 | Verified / Active |
| `mc-protocol` | 236 | Verified / Active |
| `mc-worldgen` | 331 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
