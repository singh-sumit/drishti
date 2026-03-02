---
title: Grafana Integration
sidebar_position: 2
---

Drishti ships Grafana provisioning and dashboards under `grafana/`.

## Provisioning Paths

- `grafana/provisioning/datasources/prometheus.yaml`
- `grafana/provisioning/dashboards/dashboards.yaml`
- `grafana/dashboards/overview.json`
- `grafana/dashboards/process.json`

## Local Stack

```bash
docker compose -f deploy/docker-compose.yml up -d
```

## Dashboard Expectations

- overview dashboard: CPU, memory, network, disk health summaries
- process dashboard: per-process hotspots including syscall and resource trends

If panels are empty, validate Prometheus target health and metric names from `/metrics`.
