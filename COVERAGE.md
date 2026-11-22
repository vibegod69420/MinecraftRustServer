# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 771 / 7163 (10.764%)
- **Mapped Methods**: 3100 / 66676 (4.649%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1098 | Verified / Active |
| `mc-nbt` | 1032 | Verified / Active |
| `mc-protocol` | 842 | Verified / Active |
| `mc-worldgen` | 755 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
