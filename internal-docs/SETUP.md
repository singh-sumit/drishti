# Development Setup

## Required
- Rust stable toolchain (`rustup toolchain install stable`)
- `rustfmt`, `clippy`

## For eBPF build path
- Rust nightly with `rust-src`
- `bpf-linker`
- `clang`, `llvm`, `lld`

## Optional (integration stack)
- Docker / Docker Compose for Prometheus + Grafana
- QEMU for cross-arch runtime testing

## Commands
```bash
just fmt-check
just lint
just test
cargo run -p xtask -- build-ebpf
just docs-verify
scripts/sync_github_issues.sh --repo <owner/repo> --input internal-docs/issues/backlog.yaml --dry-run
```

## Syscall Collector Notes
- `collectors.syscall.enabled` defaults to `false` to avoid unnecessary overhead on high-syscall workloads.
- Enable syscall metrics with:
  - `DRISHTI_COLLECTORS__SYSCALL=true`
  - `DRISHTI_COLLECTORS__SYSCALL__TOP_N=20`
  - `DRISHTI_COLLECTORS__SYSCALL__LATENCY_BUCKETS_USEC=1,10,50,100,500,1000,5000`
