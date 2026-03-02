---
title: Prometheus Integration
sidebar_position: 1
---

Configure Prometheus to scrape Drishti's exporter endpoint.

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: drishti
    static_configs:
      - targets: ["127.0.0.1:9090"]
```

## Validation

1. Start daemon.
2. Verify `/targets` in Prometheus UI shows Drishti as up.
3. Query `drishti_loader_failures_total` and `drishti_series_dropped_total`.

## Alerting Baseline

- non-zero `drishti_loader_failures_total`
- sustained growth in `drishti_series_dropped_total`
- scrape failures on job `drishti`
