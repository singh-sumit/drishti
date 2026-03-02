---
title: QEMU CI
sidebar_position: 3
---

## Purpose

The v0.4 QEMU lane validates end-to-end Drishti behavior in a virtualized Linux guest with the privileged eBPF runtime path enabled.

## Prerequisites

```bash
rustup component add rust-src --toolchain nightly
rustup target add aarch64-unknown-linux-musl
cargo +nightly install bpf-linker
```

System dependencies (Ubuntu):

```bash
sudo apt-get install -y qemu-system qemu-system-aarch64 cpio busybox-static clang llvm lld
```

## Local Commands

```bash
cargo run -p xtask -- qemu prepare --arch x86_64
cargo run -p xtask -- qemu smoke --arch x86_64
cargo run -p xtask -- qemu ci --arch x86_64
```

For arm64 lanes:

```bash
export DRISHTI_QEMU_AARCH64_KERNEL=/abs/path/to/vmlinuz
cargo run -p xtask -- qemu smoke --arch aarch64 --kvm off
```

If x86_64 host kernels are not available under `/boot` (for example WSL), `qemu prepare`
automatically falls back to downloading and extracting the Ubuntu generic kernel package.

## CI Trigger Policy

`qemu-ci` workflow uses hybrid gating:

- Pull Requests: `x86_64` lane
- `main` branch pushes: `x86_64` + `aarch64`
- Manual dispatch: selectable architecture (`all`, `x86_64`, `aarch64`)

## Artifacts

Per-architecture artifacts are written under `target/qemu/<arch>/` and uploaded in CI:

- `serial.log`
- `smoke.log`
- `metrics.prom`
- `summary.json`
- `index.json`

`summary.json` is the source of truth for smoke result and failure category.
