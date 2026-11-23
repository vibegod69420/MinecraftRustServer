# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 964 / 7163 (13.458%)
- **Mapped Methods**: 3898 / 66676 (5.846%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1424 | Verified / Active |
| `mc-nbt` | 1157 | Verified / Active |
| `mc-protocol` | 1198 | Verified / Active |
| `mc-worldgen` | 1009 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
