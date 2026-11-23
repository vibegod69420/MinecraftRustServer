# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 956 / 7163 (13.346%)
- **Mapped Methods**: 3843 / 66676 (5.764%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1392 | Verified / Active |
| `mc-nbt` | 1157 | Verified / Active |
| `mc-protocol` | 1198 | Verified / Active |
| `mc-worldgen` | 961 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
