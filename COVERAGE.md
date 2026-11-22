# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 613 / 7163 (8.558%)
- **Mapped Methods**: 2410 / 66676 (3.615%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 947 | Verified / Active |
| `mc-nbt` | 739 | Verified / Active |
| `mc-protocol` | 606 | Verified / Active |
| `mc-worldgen` | 582 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
