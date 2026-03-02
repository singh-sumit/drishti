---
title: Troubleshooting
sidebar_position: 1
---

## Daemon Fails to Start

Check:

1. TOML syntax and `--validate-config` output.
2. bind errors on `metrics_addr` (`Address already in use`).
3. permission errors for `pid_file` path.

## Missing Metrics

1. Verify collector toggles in config.
2. Verify synthetic mode for local deterministic testing.
3. Check `drishti_loader_failures_total` for probe attach failures.

## High Cardinality Drops

If `drishti_series_dropped_total` increases:

1. lower workload label surface (filters, fewer tracked interfaces/devices)
2. reduce syscall cardinality with smaller `collectors.syscall.top_n`
3. increase `export.max_series` only after capacity review

## CI Integration Failures

- unprivileged integration tests should pass without privileged eBPF attach
- privileged smoke tests require `DRISHTI_PRIVILEGED_TESTS=1` and capable host kernel
