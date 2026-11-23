# Reference Coverage Matrix (`COVERAGE.md`)

- **Target**: Minecraft Java Edition `1.21.11`
- **Mapped Classes**: 1462 / 7163 (20.410%)
- **Mapped Methods**: 6741 / 66676 (10.110%)

## Crate Breakdown
| Crate | Mapped Methods | Status |
|---|---:|---|
| `mc-data` | 6 | Verified / Active |
| `mc-math` | 2067 | Verified / Active |
| `mc-nbt` | 1604 | Verified / Active |
| `mc-protocol` | 2992 | Verified / Active |
| `mc-worldgen` | 1831 | Verified / Active |

## Explicit Exclusions
- `net.minecraft.client.*` (client rendering, audio, LWJGL, GLFW, client GUI) — server-only target.
- Wall-clock server watchdog crash behavior (`ServerWatchdog`) — replaced by deterministic tick stepping in oracle mode.
