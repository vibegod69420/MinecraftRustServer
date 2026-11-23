# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1543 / 7163 (21.541%)
- **Mapped Methods**: 7185 / 66676 (10.776%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2067 | Verified / Active |
| `mc-nbt` | 1656 | Verified / Active |
| `mc-protocol` | 3372 | Verified / Active |
| `mc-worldgen` | 1902 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
