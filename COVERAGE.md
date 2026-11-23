# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 818 / 7163 (11.420%)
- **Mapped Methods**: 3290 / 66676 (4.934%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1182 | Verified / Active |
| `mc-nbt` | 1098 | Verified / Active |
| `mc-protocol` | 882 | Verified / Active |
| `mc-worldgen` | 818 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
