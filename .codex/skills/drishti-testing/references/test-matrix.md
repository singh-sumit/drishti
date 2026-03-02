# Test Matrix

## Unit tests
- `config.rs`: defaults + env overrides.
- `procfs.rs`: stat/statm/meminfo parsers.
- `aggregator.rs`: label normalization + cardinality drop behavior.

## Integration tests
- Daemon starts and serves `/healthz` and `/metrics`.
- Synthetic stream updates `drishti_cpu_*`, `drishti_proc_*`, and memory metrics.
- Disabled collectors suppress corresponding metric families.

## Privileged smoke tests
- Feature-gated eBPF loader path attempts program load/attach.
- Skip unless `DRISHTI_PRIVILEGED_TESTS=1`.
