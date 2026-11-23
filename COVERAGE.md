# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1373 / 7163 (19.168%)
- **Mapped Methods**: 6168 / 66676 (9.251%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2009 | Verified / Active |
| `mc-nbt` | 1604 | Verified / Active |
| `mc-protocol` | 2538 | Verified / Active |
| `mc-worldgen` | 1612 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
