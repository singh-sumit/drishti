# Development Setup

## Required
- Rust stable toolchain (`rustup toolchain install stable`)
- `rustfmt`, `clippy`

## For eBPF build path
- Rust nightly with `rust-src`
- `bpf-linker`
- `clang`, `llvm`, `lld`
- `rustup target add aarch64-unknown-linux-musl` (needed for arm64 QEMU lane)

## Optional (integration stack)
- Docker / Docker Compose for Prometheus + Grafana
- QEMU for cross-arch runtime testing

## Commands
```bash
just fmt-check
just lint
just test
cargo run -p xtask -- build-ebpf
cargo run -p xtask -- qemu prepare --arch x86_64
cargo run -p xtask -- qemu smoke --arch x86_64
just docs-verify
scripts/sync_github_issues.sh --repo <owner/repo> --input internal-docs/issues/backlog.yaml --dry-run
```

## QEMU Notes
- x86_64 lane uses host `/boot/vmlinuz-*` when available and falls back to Ubuntu kernel package extraction when `/boot` is empty (for example on WSL).
- aarch64 lane requires explicit kernel input:
  - `DRISHTI_QEMU_AARCH64_KERNEL=/abs/path/to/vmlinuz`
  - or `DRISHTI_QEMU_AARCH64_KERNEL_DEB=/abs/path/to/linux-image-arm64.deb`
- QEMU artifacts are emitted under `target/qemu/<arch>/`.
- If arm64 build fails with missing target, run: `rustup target add aarch64-unknown-linux-musl`.

## Syscall Collector Notes
- `collectors.syscall.enabled` defaults to `false` to avoid unnecessary overhead on high-syscall workloads.
- Enable syscall metrics with:
  - `DRISHTI_COLLECTORS__SYSCALL=true`
  - `DRISHTI_COLLECTORS__SYSCALL__TOP_N=20`
  - `DRISHTI_COLLECTORS__SYSCALL__LATENCY_BUCKETS_USEC=1,10,50,100,500,1000,5000`
