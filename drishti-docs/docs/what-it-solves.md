---
title: What It Solves
sidebar_position: 2
---

Drishti solves a practical problem: getting process-level Linux observability without modifying applications.

## Primary Problems

1. Missing runtime visibility into CPU, memory, network, disk, and syscall behavior.
2. Inconsistent data pipelines across teams and environments.
3. High operational cost for ad hoc per-host debugging.

## Drishti Approach

- kernel-side probes emit bounded eBPF events
- daemon collectors aggregate and normalize labels
- Prometheus `/metrics` endpoint exposes `drishti_*` series
- Grafana dashboards visualize host and process behavior

## Operational Outcomes

- faster root-cause analysis for regression and saturation incidents
- a single telemetry surface for local dev, CI smoke, and production hosts
- explicit cardinality controls through `max_series` and syscall `top_n`

## Constraints

Drishti intentionally targets Linux 5.8+ and treats eBPF probe attach as best-effort so the daemon can run with partial collector availability.
