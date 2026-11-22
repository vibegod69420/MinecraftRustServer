# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 501 / 7163 (6.994%)
- **Mapped Methods**: 1722 / 66676 (2.583%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 693 | Verified / Active |
| `mc-nbt` | 496 | Verified / Active |
| `mc-protocol` | 357 | Verified / Active |
| `mc-worldgen` | 516 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
