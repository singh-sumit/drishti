---
title: Drishti Engineering Docs
sidebar_position: 1
---

Drishti is a Rust observability daemon that combines eBPF signal collection with a Prometheus exporter and Grafana dashboards.

This documentation set is focused on engineering and operations:

- architecture and data-path decisions
- deployment and integration guidance
- collector and metrics contracts
- performance and troubleshooting playbooks

## Audience

- platform engineers operating Linux fleets
- SRE teams integrating Prometheus/Grafana
- developers extending collectors and eBPF hooks

## Scope

Current milestone coverage includes:

- CPU and process lifecycle telemetry
- memory collector from procfs + OOM event path
- network telemetry (tx/rx, RTT, retransmits)
- disk telemetry (bytes, IOPS, latency, queue depth)
- syscall tracing (count, error, latency)

## Start Here

1. [What It Solves](./what-it-solves.md)
2. [Architecture Overview](./architecture/overview.md)
3. [Quickstart](./usage/quickstart.md)
4. [Configuration](./usage/configuration.md)
5. [Metrics Reference](./reference/metrics.md)

## Internal Project Docs

Non-published engineering artifacts remain in the repository under `internal-docs/`:

- `internal-docs/aether-ebpf-observability-spec.docx`
- `internal-docs/issues/backlog.yaml`
- `internal-docs/SETUP.md`
