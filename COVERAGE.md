# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 451 / 7163 (6.296%)
- **Mapped Methods**: 1536 / 66676 (2.304%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 606 | Verified / Active |
| `mc-nbt` | 480 | Verified / Active |
| `mc-protocol` | 309 | Verified / Active |
| `mc-worldgen` | 446 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
