# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 559 / 7163 (7.804%)
- **Mapped Methods**: 2036 / 66676 (3.054%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 778 | Verified / Active |
| `mc-nbt` | 614 | Verified / Active |
| `mc-protocol` | 490 | Verified / Active |
| `mc-worldgen` | 540 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
