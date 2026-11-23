# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1143 / 7163 (15.957%)
- **Mapped Methods**: 4847 / 66676 (7.269%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 1731 | Verified / Active |
| `mc-nbt` | 1417 | Verified / Active |
| `mc-protocol` | 1624 | Verified / Active |
| `mc-worldgen` | 1283 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
