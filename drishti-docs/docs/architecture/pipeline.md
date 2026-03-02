---
title: Telemetry Pipeline
sidebar_position: 2
---

This sequence shows the end-to-end telemetry path from kernel event to scrape.

```mermaid
sequenceDiagram
  participant K as Kernel Hook
  participant B as eBPF Program
  participant R as RingBuf/Map
  participant D as Daemon Collector
  participant A as Aggregator
  participant E as Exporter
  participant P as Prometheus

  K->>B: tracepoint/kprobe fires
  B->>R: write bounded event/map update
  D->>R: poll ring buffer + read maps
  D->>A: record event with labels
  A->>A: enforce max_series/top_n
  E->>A: encode OpenMetrics
  P->>E: GET /metrics
  E-->>P: drishti_* text exposition
```

## Collector Isolation

Collectors run as independent tasks, so one noisy source does not block all telemetry.

## Health Contract

`GET /healthz` returns `200` while the exporter loop is active.
