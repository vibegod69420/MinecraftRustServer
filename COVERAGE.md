# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 530 / 7163 (7.399%)
- **Mapped Methods**: 1857 / 66676 (2.785%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 724 | Verified / Active |
| `mc-nbt` | 548 | Verified / Active |
| `mc-protocol` | 420 | Verified / Active |
| `mc-worldgen` | 523 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
