# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1362 / 7163 (19.014%)
- **Mapped Methods**: 6101 / 66676 (9.150%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2009 | Verified / Active |
| `mc-nbt` | 1604 | Verified / Active |
| `mc-protocol` | 2458 | Verified / Active |
| `mc-worldgen` | 1593 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
