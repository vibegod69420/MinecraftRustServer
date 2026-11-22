# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 514 / 7163 (7.176%)
- **Mapped Methods**: 1802 / 66676 (2.703%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 722 | Verified / Active |
| `mc-nbt` | 518 | Verified / Active |
| `mc-protocol` | 400 | Verified / Active |
| `mc-worldgen` | 516 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
