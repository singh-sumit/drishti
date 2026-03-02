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
```
