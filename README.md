# Drishti

Drishti is a Rust observability daemon that combines eBPF event collection with a Prometheus-compatible exporter and Grafana dashboards.

## Docs

Published engineering docs: <https://singh-sumit.github.io/drishti/>

Local docs source:

- published docs site: `drishti-docs/`
- internal engineering docs and planning assets: `internal-docs/`

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

One-shot deterministic output:

```bash
cargo run -p drishti-daemon -- --config config/drishti.toml --once
```

## Local Development Commands

```bash
just fmt-check
just lint
just test
```

## Docs Development Commands

```bash
just docs-install
just docs-dev
just docs-build
just docs-verify
```

## GitHub Issue Sync

```bash
scripts/sync_github_issues.sh --repo <owner/repo> --input internal-docs/issues/backlog.yaml --dry-run
scripts/sync_github_issues.sh --repo <owner/repo> --input internal-docs/issues/backlog.yaml --apply
```

The sync flow creates/updates milestones and issues, maintains deterministic parent tasklists from `parent_id`, and closes issues where `status=done`.
