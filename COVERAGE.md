# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1365 / 7163 (19.056%)
- **Mapped Methods**: 6131 / 66676 (9.195%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2009 | Verified / Active |
| `mc-nbt` | 1604 | Verified / Active |
| `mc-protocol` | 2495 | Verified / Active |
| `mc-worldgen` | 1603 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
