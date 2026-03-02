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
- QEMU validation workflow (`.github/workflows/qemu-ci.yml`)
- privileged smoke tests remain gated by environment and host capability

## QEMU Validation

The QEMU lane validates guest boot, loader attach path, exporter readiness, and expected
`drishti_*` metric families through the `qemu_smoke` harness.

Triggers:

- pull requests: `x86_64` lane
- `main` pushes: `x86_64` and `aarch64` lanes
- manual dispatch: choose architecture via workflow input

Local host note: `x86_64` smoke is expected to run directly after toolchain setup.
`aarch64` smoke additionally requires `rustup target add aarch64-unknown-linux-musl`
and an explicit arm64 kernel path or kernel `.deb` input.
