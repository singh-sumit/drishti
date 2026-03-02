---
title: API Reference
sidebar_position: 2
---

Drishti's public runtime API is HTTP-based and intentionally small.

## Endpoints

- `GET /metrics`: Prometheus/OpenMetrics exposition.
- `GET /healthz`: readiness/liveness indicator for exporter loop.

## CLI Surface

- `--config <path>`
- `--validate-config`
- `--once`
- `--log-format <text|json>`

## Rust API Docs

Rust API docs are generated from source and published separately from Docusaurus.

Generate locally:

```bash
cargo doc --workspace --no-deps
```

Open generated docs from `target/doc/` for crate-level interfaces.
