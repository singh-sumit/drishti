---
title: Security and Capabilities
sidebar_position: 3
---

Drishti is designed to run with least privilege while still supporting eBPF attach where allowed.

## Capability Model

- privileged eBPF path: requires kernel features and attach permissions
- unprivileged test path: synthetic events + procfs collector for deterministic CI
- systemd hardening: capability bounding and filesystem restrictions in unit config

## Deployment Topology

```mermaid
flowchart TB
  subgraph Host[Linux Host]
    SD[systemd unit\ndrishti-daemon.service]
    DD[drishti-daemon]
    SD --> DD
  end

  DD -->|:9090 /metrics| PR[Prometheus]
  PR --> GF[Grafana]

  subgraph Optional[Optional Validation Lane]
    Q[QEMU VM]
    T[Privileged smoke tests]
    Q --> T
  end

  T -. validates .-> DD
```

## Safety Controls

- bounded map capacities in eBPF programs
- best-effort attach with explicit warnings
- config-level collector toggles for overhead control
- syscall `top_n` collapse to reduce label explosion
