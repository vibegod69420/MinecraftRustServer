# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 462 / 7163 (6.450%)
- **Mapped Methods**: 1575 / 66676 (2.362%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 623 | Verified / Active |
| `mc-nbt` | 494 | Verified / Active |
| `mc-protocol` | 309 | Verified / Active |
| `mc-worldgen` | 459 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
