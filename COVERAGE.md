# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 828 / 7163 (11.559%)
- **Mapped Methods**: 3327 / 66676 (4.990%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1208 | Verified / Active |
| `mc-nbt` | 1106 | Verified / Active |
| `mc-protocol` | 891 | Verified / Active |
| `mc-worldgen` | 820 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
