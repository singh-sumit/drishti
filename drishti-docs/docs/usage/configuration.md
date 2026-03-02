---
title: Configuration
sidebar_position: 2
---

Drishti reads TOML config plus environment overrides using `DRISHTI_` and double underscore separators.

## Example

```toml
[daemon]
pid_file = "/var/run/drishti.pid"
log_level = "info"
metrics_addr = "0.0.0.0:9090"

[collectors]
cpu = true

[collectors.process]
enabled = true
track_threads = false

[collectors.memory]
enabled = true
poll_interval_ms = 1000
track_oom = true

[collectors.network]
enabled = true
interfaces = []
tcp_rtt = true
tcp_retransmits = true

[collectors.disk]
enabled = true
devices = []
latency_buckets_usec = [10, 50, 100, 500, 1000, 5000, 10000]

[collectors.syscall]
enabled = false
top_n = 20
latency_buckets_usec = [1, 10, 50, 100, 500, 1000, 5000]

[filters]
exclude_pids = [1, 2]
exclude_comms = ["kworker"]
include_comms = []

[export]
scrape_interval_ms = 1000
max_series = 10000
```

## Environment Override Examples

```bash
export DRISHTI_DAEMON__LOG_LEVEL=debug
export DRISHTI_COLLECTORS__SYSCALL=true
export DRISHTI_COLLECTORS__SYSCALL__TOP_N=30
export DRISHTI_EXPORT__MAX_SERIES=12000
```

## Notes

- syscall collector is disabled by default due event volume.
- network and disk collectors support include-lists (`interfaces`, `devices`).
- cardinality controls are enforced globally with `max_series`.
