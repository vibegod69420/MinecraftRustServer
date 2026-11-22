# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 637 / 7163 (8.893%)
- **Mapped Methods**: 2504 / 66676 (3.756%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 949 | Verified / Active |
| `mc-nbt` | 766 | Verified / Active |
| `mc-protocol` | 644 | Verified / Active |
| `mc-worldgen` | 630 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
