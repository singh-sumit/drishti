# Drishti

Drishti is a Rust observability daemon that combines eBPF event collection with a Prometheus-compatible exporter and Grafana dashboards.

## Workspace
- `drishti-common`: shared ABI-safe event/map types
- `drishti-ebpf`: kernel-side eBPF programs
- `drishti-daemon`: user-space daemon and metrics exporter
- `xtask`: build and maintenance tasks

## Quick Start
```bash
just build
cargo run -p drishti-daemon -- --config config/drishti.toml
```

## Common Commands
```bash
just fmt-check
just lint
just test
cargo run -p xtask -- build-ebpf
```

## GitHub Issue Sync
```bash
scripts/sync_github_issues.sh --repo <owner/repo> --input docs/issues/backlog.yaml --dry-run
```
