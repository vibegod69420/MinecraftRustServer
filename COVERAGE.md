# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 752 / 7163 (10.498%)
- **Mapped Methods**: 3006 / 66676 (4.508%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1090 | Verified / Active |
| `mc-nbt` | 1002 | Verified / Active |
| `mc-protocol` | 783 | Verified / Active |
| `mc-worldgen` | 740 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
