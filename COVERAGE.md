# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 587 / 7163 (8.195%)
- **Mapped Methods**: 2219 / 66676 (3.328%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 938 | Verified / Active |
| `mc-nbt` | 626 | Verified / Active |
| `mc-protocol` | 528 | Verified / Active |
| `mc-worldgen` | 569 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
