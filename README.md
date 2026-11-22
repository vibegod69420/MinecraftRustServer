# Minecraft 1.21.11 Rust Server (`mc-rust`)

A clean-room, faithful Rust port of the Minecraft Java Edition `1.21.11` server, built via a two-phase differential verification strategy:
1. **Phase A (Faithful Port)**: Exact 1:1 behavioral parity (algorithms, RNG sequences, float operation order, iteration order, NBT/Anvil formats, protocol packets).
2. **Phase B (Optimization)**: Measured CPU/RAM/I/O optimizations targeting smooth operation on `1 vCPU / ~1 GB RAM`, gated by the full differential verification suite.

## Repository Layout
- `crates/` — Rust workspace crates (`mc-math`, `mc-nbt`, `mc-data`, `mc-protocol`, `mc-worldgen`, `mc-harness`)
- `oracle/` — Deterministic Java agent (ASM) & vanilla server test driver
- `harness/` — Rust differential runner, Merkle comparator, shrinker configs
- `scenarios/` — Deterministic scenario definitions
- `infra/` — AWS CloudFormation template, bootstrap script, and management CLI (`manage.ps1`)
- `scripts/` — Reference downloader/decompiler, codegen, mapping checker
- `reference/` — **(Gitignored / Private)** Official Mojang jars, mappings, generated data reports, Vineflower decompiled sources, golden vectors

## Quick Start

### 1. Fetch Reference Sources & Run Vanilla Data Generator
```powershell
python scripts/fetch_reference.py
```

### 2. Verify Deterministic Oracle Self-Test (Step 0)
```powershell
python oracle/run_oracle.py --self-test
```

### 3. Generate Data & Golden Vectors, Then Build & Test Rust Crates
```powershell
python scripts/codegen_data.py
cargo test --workspace
python scripts/check_mapping.py
```

### 4. AWS Infrastructure (`mc-rust-dev`)
```powershell
.\infra\manage.ps1 status
.\infra\manage.ps1 start
.\infra\manage.ps1 stop
```
