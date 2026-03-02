# Workspace Map

- `drishti-common`: shared ABI-safe event and map types.
- `drishti-ebpf`: kernel-side probes and map declarations for BPF target builds.
- `drishti-daemon`: config, loader, collectors, aggregation, exporter.
- `xtask`: developer automation (`build-ebpf`, skill validation).

When adding a metric, update in this order:
1. event emission source (eBPF or procfs collector)
2. collector event handling
3. aggregator metric registration + updates
4. integration assertions
5. dashboard query
