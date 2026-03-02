# Drishti

Drishti is a Rust observability daemon that combines eBPF event collection with a Prometheus-compatible exporter and Grafana dashboards.

v0.2 expands collector coverage to CPU, process lifecycle, memory, network, and disk telemetry with `drishti_*` Prometheus metrics.

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

Run a one-shot synthetic collection for quick verification:

```bash
cargo run -p drishti-daemon -- --config config/drishti.toml --once
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
scripts/sync_github_issues.sh --repo <owner/repo> --input docs/issues/backlog.yaml --apply
```

The sync flow now creates/updates milestones and issues, and writes deterministic parent tasklists from `parent_id` relationships in `docs/issues/backlog.yaml`.
