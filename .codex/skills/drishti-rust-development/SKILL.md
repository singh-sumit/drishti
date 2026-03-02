---
name: drishti-rust-development
description: Implement and evolve Rust code in the Drishti workspace (drishti-common, drishti-ebpf, drishti-daemon, xtask). Use when changing collectors, metrics, config, loader behavior, workspace/build files, or Rust architecture decisions.
---

# Drishti Rust Development

## Workflow
1. Locate affected crate boundaries before editing; keep shared ABI in `drishti-common`.
2. Keep event-path changes compatible between loader, collector handlers, and metric aggregation.
3. Keep eBPF changes bounded and verifier-safe; gate runtime attach behavior behind feature flags.
4. Update configs, docs, and deploy assets when interfaces change.
5. Run `scripts/dev_loop.sh` and fix warnings before finalizing.

## Design Rules
- Keep `#[repr(C)]` structs stable and copy-only.
- Keep metric names prefixed with `drishti_`.
- Keep per-process labels sanitized and cardinality bounded.
- Keep fallbacks operational when eBPF runtime dependencies are missing.

## References
- Use `references/workspace-map.md` for crate responsibilities.
