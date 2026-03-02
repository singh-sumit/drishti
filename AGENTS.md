# Drishti Agent Rules

## Mission
Ship safe, testable Rust changes for the Drishti observability workspace, with special care around eBPF verifier constraints, metric cardinality, and runtime overhead.

## Coding Standards
- Keep modules small and cohesive.
- Prefer explicit error contexts (`anyhow::Context`) at I/O and OS boundaries.
- Keep `unsafe` blocks narrowly scoped with a short safety comment.
- Keep shared ABI structs in `drishti-common` as `#[repr(C)]` and copy-only.
- Avoid hidden allocations in hot event paths.

## eBPF Safety
- Keep loops bounded.
- Avoid unchecked pointer arithmetic.
- Keep map sizes bounded and justified.
- Treat tracepoint attach as best-effort: log failures and continue with partial functionality.

## Metrics and Labels
- Prefix every metric with `drishti_`.
- Use stable label keys and sanitize process names.
- Enforce cardinality via `max_series`; count dropped series explicitly.

## Testing and Validation
- Add unit tests for parsers, config, and aggregation logic.
- Keep unprivileged integration tests deterministic and CI-required.
- Gate privileged eBPF smoke tests with `DRISHTI_PRIVILEGED_TESTS=1`.
- Before merge, run: `just fmt-check`, `just lint`, `just test`.

## Issue Workflow
- Keep local backlog in `docs/issues/backlog.yaml`.
- Regenerate issue markdown under `docs/issues/generated/` after edits.
- Use `scripts/sync_github_issues.sh --dry-run` before `--apply`.
