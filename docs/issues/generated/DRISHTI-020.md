# Implement shared ABI-safe event and map types

Local ID: `DRISHTI-020`
Type: `task`
Status: `done`
Milestone: `v0.1 Drishti Core`
Labels: `rust, ebpf, v0.1`
Depends on: `DRISHTI-010`

Define `#[repr(C)]` event and map structs in `drishti-common` for CPU, process lifecycle, and OOM paths.
