# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 464 / 7163 (6.478%)
- **Mapped Methods**: 1587 / 66676 (2.380%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 636 | Verified / Active |
| `mc-nbt` | 494 | Verified / Active |
| `mc-protocol` | 309 | Verified / Active |
| `mc-worldgen` | 459 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
