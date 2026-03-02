---
title: Metrics Reference
sidebar_position: 1
---

All metrics are exported with `drishti_` prefix.

## CPU and Process

- `drishti_cpu_run_time_ns_total`
- `drishti_cpu_wait_time_ns_total`
- `drishti_proc_lifecycle_total`

## Memory

- `drishti_mem_rss_bytes`
- `drishti_mem_vss_bytes`
- `drishti_mem_page_faults_minor_total`
- `drishti_mem_page_faults_major_total`
- `drishti_mem_oom_kills_total`
- `drishti_mem_available_bytes`
- `drishti_mem_cache_bytes`
- `drishti_mem_total_bytes`

## Network

- `drishti_net_tx_bytes_total`
- `drishti_net_rx_bytes_total`
- `drishti_net_tx_packets_total`
- `drishti_net_rx_packets_total`
- `drishti_net_tcp_rtt_usec`
- `drishti_net_tcp_retransmits_total`

## Disk

- `drishti_disk_read_bytes_total`
- `drishti_disk_write_bytes_total`
- `drishti_disk_iops_total`
- `drishti_disk_io_latency_usec`
- `drishti_disk_queue_depth`

## Syscall

- `drishti_syscall_count_total`
- `drishti_syscall_error_total`
- `drishti_syscall_latency_usec`

## Daemon Internal

- `drishti_collect_scrape_duration_ms`
- `drishti_series_dropped_total`
- `drishti_loader_failures_total`
