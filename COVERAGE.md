# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 2188 / 7163 (30.546%)
- **Mapped Methods**: 9695 / 66676 (14.540%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 4163 | Verified / Active |
| `mc-nbt` | 1915 | Verified / Active |
| `mc-protocol` | 3587 | Verified / Active |
| `mc-worldgen` | 2648 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
