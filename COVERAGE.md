# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 336 / 7163 (4.691%)
- **Mapped Methods**: 1086 / 66676 (1.629%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 548 | Verified / Active |
| `mc-nbt` | 218 | Verified / Active |
| `mc-protocol` | 236 | Verified / Active |
| `mc-worldgen` | 324 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
