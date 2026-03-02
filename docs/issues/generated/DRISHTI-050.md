# Implement memory procfs collector and filters

Local ID: `DRISHTI-050`
Type: `task`
Status: `done`
Labels: `memory, procfs, v0.1`
Depends on: `DRISHTI-040`

Poll `/proc` for per-process RSS/VSS/fault data and system memory totals, respecting include/exclude filters.
