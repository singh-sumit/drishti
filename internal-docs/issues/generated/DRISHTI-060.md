# Add Prometheus exporter and metric aggregation guards

Local ID: `DRISHTI-060`
Type: `task`
Status: `done`
Milestone: `v0.1 Drishti Core`
Labels: `metrics, prometheus, v0.1`
Depends on: `DRISHTI-040, DRISHTI-050`

Expose `/metrics` and `/healthz`, register `drishti_*` metrics, and enforce `max_series` cardinality circuit breaker.

## Dependency Links
- #5
- #6
