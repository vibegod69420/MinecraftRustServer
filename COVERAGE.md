# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 743 / 7163 (10.373%)
- **Mapped Methods**: 2973 / 66676 (4.459%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1084 | Verified / Active |
| `mc-nbt` | 1002 | Verified / Active |
| `mc-protocol` | 783 | Verified / Active |
| `mc-worldgen` | 704 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
