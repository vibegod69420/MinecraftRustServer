# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 406 / 7163 (5.668%)
- **Mapped Methods**: 1403 / 66676 (2.104%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 588 | Verified / Active |
| `mc-nbt` | 402 | Verified / Active |
| `mc-protocol` | 309 | Verified / Active |
| `mc-worldgen` | 391 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
