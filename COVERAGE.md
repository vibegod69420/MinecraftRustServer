# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 412 / 7163 (5.752%)
- **Mapped Methods**: 1418 / 66676 (2.127%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 591 | Verified / Active |
| `mc-nbt` | 402 | Verified / Active |
| `mc-protocol` | 309 | Verified / Active |
| `mc-worldgen` | 408 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
