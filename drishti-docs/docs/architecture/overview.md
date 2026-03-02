---
title: Architecture Overview
sidebar_position: 1
---

Drishti uses a layered model: shared ABI types, kernel probes, daemon collectors, aggregation, and exporter.

```mermaid
flowchart LR
  K["Linux Kernel<br/>tracepoints/kprobes"] --> E["eBPF Programs<br/>drishti-ebpf"]
  E -->|"ring buffer events"| C["Collectors<br/>drishti-daemon"]
  E -->|"map snapshots"| C
  C --> A["Aggregator<br/>metric families + cardinality guard"]
  A --> X["Exporter<br/>/metrics + /healthz"]
  X --> P[Prometheus]
  P --> G[Grafana Dashboards]

  S["drishti-common<br/>repr C event/map ABI"] --> E
  S --> C
```

## Workspace Contracts

- `drishti-common`: `#[repr(C)]` shared event/map structs.
- `drishti-ebpf`: kernel-side programs with bounded maps and verifier-safe logic.
- `drishti-daemon`: loader, collectors, aggregation, and HTTP exporter.
- `xtask`: build helper path for eBPF artifacts.

## Runtime Guarantees

- partial probe attach failures are non-fatal and explicitly logged
- metrics stay prefixed with `drishti_`
- high-cardinality series are dropped and counted via `drishti_series_dropped_total`
