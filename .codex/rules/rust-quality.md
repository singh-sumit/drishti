# Rust Quality Policy

## Error Handling
- Return `Result` from fallible boundaries.
- Add context strings for filesystem, process, and socket operations.
- Avoid `unwrap()` in non-test code.

## Unsafe Boundaries
- Keep `unsafe` in dedicated helper functions where possible.
- Explain preconditions for each unsafe block in one short comment.

## eBPF Integration
- Keep eBPF runtime feature-gated for predictable local development.
- Keep fallback paths testable when eBPF toolchain is unavailable.

## Testing Minimums
- Every parser must have positive and negative tests.
- Every public config override path must have coverage.
- Every new metric family must appear in at least one integration assertion.

## Performance Guardrails
- Prefer counters/gauges over high-cardinality histograms unless justified.
- Avoid per-event heap allocation in hot loops.
