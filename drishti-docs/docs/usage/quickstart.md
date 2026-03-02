---
title: Quickstart
sidebar_position: 1
---

## Prerequisites

- Linux host
- Rust toolchain installed
- optional: eBPF build prerequisites for privileged probe path (`clang`, `llvm`, `bpf-linker`)

## Build and Run

```bash
just build
cargo run -p drishti-daemon -- --config config/drishti.toml
```

Run one-shot collection for local verification:

```bash
cargo run -p drishti-daemon -- --config config/drishti.toml --once
```

Validate config only:

```bash
cargo run -p drishti-daemon -- --config config/drishti.toml --validate-config
```

## Check Endpoints

```bash
curl -s http://127.0.0.1:9090/healthz
curl -s http://127.0.0.1:9090/metrics
```

## CLI Flags

- `--config <path>` (default: `config/drishti.toml`)
- `--validate-config`
- `--once`
- `--log-format <text|json>`
