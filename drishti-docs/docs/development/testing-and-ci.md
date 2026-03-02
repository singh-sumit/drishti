---
title: Testing and CI
sidebar_position: 2
---

## Required Rust Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Docs Gates

```bash
cd drishti-docs
npm ci
npm run build
```

The docs build is configured to fail on broken links.

## CI Layout

- core Rust quality/test workflow (`.github/workflows/ci.yml`)
- docs build/deploy workflow (`.github/workflows/docs.yml`)
- privileged smoke tests remain gated by environment and host capability
