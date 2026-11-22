# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 39 / 7163 (0.544%)
- **Mapped Methods**: 163 / 66676 (0.244%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 183 | Verified / Active |
| `mc-nbt` | 6 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
