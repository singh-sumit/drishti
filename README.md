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

## Observability Stack (Local)

Bring up Drishti + Prometheus + Grafana with managed local lifecycle:

```bash
sudo -v
just obs-up
just obs-status
just obs-down
```

Endpoints:

- Drishti metrics: `http://127.0.0.1:9090/metrics`
- Prometheus: `http://127.0.0.1:9091`
- Grafana: `http://127.0.0.1:3000` (`admin/admin`)

`just obs-up` enables the syscall collector via `DRISHTI_COLLECTORS__SYSCALL=true` for richer dashboards.

`just obs-up` also builds and runs the daemon with `ebpf-runtime` and an embedded eBPF artifact, so
kernel-event-driven syscall series are available without extra manual build steps.
For local churn-heavy workloads, it defaults `DRISHTI_EXPORT__MAX_SERIES=50000` (override by exporting
`DRISHTI_EXPORT__MAX_SERIES` before running `just obs-up`).

## Full eBPF Local Guide (Prometheus + Grafana)

Use this path when you want real eBPF-driven data in Grafana panels.

### 1) Install host prerequisites

```bash
sudo apt-get update
sudo apt-get install -y qemu-system cpio busybox-static clang llvm lld
rustup toolchain install nightly --profile minimal --component rust-src
cargo +nightly install bpf-linker
```

### 2) Build eBPF object + daemon with `ebpf-runtime` (manual path)

```bash
cargo run -p xtask -- build-ebpf
export DRISHTI_EMBEDDED_BPF_PATH="$(pwd)/target/bpfel-unknown-none/release/drishti-ebpf"
cargo build -p drishti-daemon --features ebpf-runtime --bin drishti-daemon
```

Optional sanity check:

```bash
file "$DRISHTI_EMBEDDED_BPF_PATH"
```

### 3) Start Prometheus and Grafana

```bash
docker compose -f deploy/docker-compose.yml up -d --remove-orphans
```

### 4) Start daemon in privileged mode

```bash
sudo -v
sudo env \
  DRISHTI_DAEMON__PID_FILE=/tmp/drishti.pid \
  DRISHTI_COLLECTORS__SYSCALL=true \
  ./target/debug/drishti-daemon --config config/drishti.toml
```

If you prefer background mode:

```bash
sudo env \
  DRISHTI_DAEMON__PID_FILE=/tmp/drishti.pid \
  DRISHTI_COLLECTORS__SYSCALL=true \
  nohup ./target/debug/drishti-daemon --config config/drishti.toml \
  > target/obs/drishti-daemon.log 2>&1 < /dev/null &
```

### 5) Validate end-to-end

```bash
curl -fsS http://127.0.0.1:9090/healthz
curl -fsS http://127.0.0.1:9090/metrics | grep '^drishti_'
curl -fsS http://127.0.0.1:9091/-/ready
curl -fsS 'http://127.0.0.1:9091/api/v1/targets?state=active'
curl -fsS 'http://127.0.0.1:9091/api/v1/query?query=drishti_syscall_count_total'
```

Expected:

- Prometheus target health is `up`.
- `drishti_*` families are present from `/metrics`.
- `drishti_syscall_*` queries return non-empty vectors when eBPF attach succeeds.

### 6) Open Grafana

- URL: `http://127.0.0.1:3000`
- Login: `admin/admin`
- Dashboards:
  - `Drishti Overview`
  - `Drishti Process Focus`
  - `Drishti Syscalls`

### 7) Stop and clean

```bash
docker compose -f deploy/docker-compose.yml down --remove-orphans
if [[ -f /tmp/drishti.pid ]]; then sudo kill "$(cat /tmp/drishti.pid)"; fi
```

## Empty Panel Troubleshooting

If Grafana panels are empty:

1. Check daemon log for `compiled without ebpf-runtime feature` (applies to manual runs or stale binaries).
2. Ensure `DRISHTI_EMBEDDED_BPF_PATH` was set during daemon build.
3. Confirm Prometheus target state is `up` at `/api/v1/targets`.
4. Verify syscall collector is enabled (`DRISHTI_COLLECTORS__SYSCALL=true`).
5. Confirm `/metrics` contains `drishti_` families before debugging Grafana panels.

## QEMU Validation (v0.4)

```bash
just qemu-prepare arch=x86_64
just qemu-smoke-x86
```

If `/boot/vmlinuz-*` is missing (common on WSL), `just qemu-prepare` now auto-falls
back to extracting an Ubuntu generic kernel package for x86_64.

For arm64 runs, provide a kernel image (or kernel .deb extract source):

```bash
rustup target add aarch64-unknown-linux-musl
DRISHTI_QEMU_AARCH64_KERNEL=/abs/path/to/vmlinuz \
  just qemu-smoke-arm64
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
