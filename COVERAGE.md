# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 865 / 7163 (12.076%)
- **Mapped Methods**: 3441 / 66676 (5.161%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1235 | Verified / Active |
| `mc-nbt` | 1113 | Verified / Active |
| `mc-protocol` | 968 | Verified / Active |
| `mc-worldgen` | 858 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
