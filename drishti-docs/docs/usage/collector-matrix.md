---
title: Collector Matrix
sidebar_position: 3
---

| Collector | Default | Source | Primary Metrics |
| --- | --- | --- | --- |
| CPU | enabled | eBPF scheduler events + synthetic fallback | `drishti_cpu_run_time_ns_total`, `drishti_cpu_wait_time_ns_total` |
| Process lifecycle | enabled | eBPF process tracepoints + synthetic fallback | `drishti_proc_lifecycle_total` |
| Memory | enabled | `/proc` polling + OOM event path | `drishti_mem_*` |
| Network | enabled | eBPF hooks + synthetic fallback | `drishti_net_*` |
| Disk | enabled | eBPF block path + synthetic fallback | `drishti_disk_*` |
| Syscall | disabled | eBPF raw syscall tracepoints + synthetic fallback | `drishti_syscall_*` |

## Deterministic CI Behavior

Unprivileged CI runs validate collectors through deterministic synthetic event streams, so integration tests remain stable without privileged kernel attach.
