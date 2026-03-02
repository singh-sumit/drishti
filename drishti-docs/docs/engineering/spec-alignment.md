---
title: Spec Alignment
sidebar_position: 1
---

This repository implementation aligns to the architecture and requirement intent in `internal-docs/aether-ebpf-observability-spec.docx`.

Reference source:

- [Aether eBPF Observability Spec](https://github.com/singh-sumit/drishti/blob/main/internal-docs/aether-ebpf-observability-spec.docx)

## Functional Alignment Snapshot

| Spec Theme | Drishti Status |
| --- | --- |
| CPU metrics and process lifecycle | Implemented |
| Memory metrics + OOM path | Implemented |
| Network telemetry | Implemented |
| Disk telemetry | Implemented |
| Syscall tracing latency/errors | Implemented |
| Prometheus `/metrics` export | Implemented |
| Grafana dashboards | Implemented (provisioned JSON dashboards) |
| systemd service mode | Implemented |
| QEMU full CI lane | Implemented (hybrid gating: x86_64 on PR, aarch64 on main/manual) |
| musl size optimization | Deferred to v0.5 issue track |

## Design Decisions Preserved

1. Rust-first stack with `aya` and `tokio`.
2. Shared ABI in `drishti-common` with `#[repr(C)]` structs.
3. Bounded maps and verifier-safe eBPF patterns.
4. Deterministic local-first issue planning with scripted GitHub sync.

## Notable Divergence

Original spec names used `aether_*` metric prefixes. Implementation intentionally uses `drishti_*` for all exported series.
