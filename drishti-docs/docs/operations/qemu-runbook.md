---
title: QEMU Runbook
sidebar_position: 3
---

## Failure Categories

Drishti QEMU smoke output uses deterministic categories in `summary.json`:

- `boot_timeout`
- `daemon_startup_failure`
- `exporter_readiness_failure`
- `missing_metrics_family`
- `attach_load_failure`
- `guest_exit_without_result`

## Quick Triage

1. Open `target/qemu/<arch>/summary.json`.
2. Inspect `target/qemu/<arch>/serial.log`.
3. Confirm metric snapshot in `target/qemu/<arch>/metrics.prom`.

## Common Issues

### Missing kernel image

- x86_64 uses host `/boot/vmlinuz-*` by default, with automatic Ubuntu package extraction fallback when `/boot` does not contain a kernel.
- aarch64 requires explicit kernel source:
  - `DRISHTI_QEMU_AARCH64_KERNEL`
  - or `DRISHTI_QEMU_AARCH64_KERNEL_DEB`

### Missing arm64 Rust target

If `xtask qemu smoke --arch aarch64` fails before build starts, install:

```bash
rustup target add aarch64-unknown-linux-musl
```

### eBPF object not embedded

If smoke reports loader attach/load failure, confirm daemon binaries were built with:

- `--features ebpf-runtime`
- `DRISHTI_EMBEDDED_BPF_PATH` pointing to compiled `drishti-ebpf` object

### KVM unavailable

Use `--kvm off` (TCG fallback).

```bash
cargo run -p xtask -- qemu smoke --arch x86_64 --kvm off
```

### Kernel panic after PASS marker

In the initramfs smoke model, `/init` exits after writing result markers. The guest kernel
can log `Kernel panic - not syncing: Attempted to kill init!` immediately afterward.
Treat `summary.json` as source of truth; panic lines after a `PASS` summary are expected.
