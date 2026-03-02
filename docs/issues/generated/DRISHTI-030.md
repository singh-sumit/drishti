# Implement eBPF scheduler and process probes

Local ID: `DRISHTI-030`
Type: `task`
Status: `done`
Milestone: `v0.1 Drishti Core`
Labels: `ebpf, cpu, process, v0.1`
Depends on: `DRISHTI-020`

Add `sched_switch`, `sched_wakeup`, and `sched_process_*` tracepoint programs with bounded maps and ring buffer output.
