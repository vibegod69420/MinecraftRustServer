# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 592 / 7163 (8.265%)
- **Mapped Methods**: 2299 / 66676 (3.448%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 944 | Verified / Active |
| `mc-nbt` | 679 | Verified / Active |
| `mc-protocol` | 559 | Verified / Active |
| `mc-worldgen` | 569 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
