# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 576 / 7163 (8.041%)
- **Mapped Methods**: 2105 / 66676 (3.157%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 814 | Verified / Active |
| `mc-nbt` | 614 | Verified / Active |
| `mc-protocol` | 528 | Verified / Active |
| `mc-worldgen` | 565 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
