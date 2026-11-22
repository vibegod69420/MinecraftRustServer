# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 245 / 7163 (3.420%)
- **Mapped Methods**: 733 / 66676 (1.099%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 435 | Verified / Active |
| `mc-nbt` | 96 | Verified / Active |
| `mc-protocol` | 155 | Verified / Active |
| `mc-worldgen` | 245 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
