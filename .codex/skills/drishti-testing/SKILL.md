---
name: drishti-testing
description: Build and maintain test coverage for Drishti (unit, integration, and gated privileged smoke tests). Use when changing parsers, metrics, collectors, daemon startup/shutdown, CI test jobs, or acceptance checks.
---

# Drishti Testing

## Workflow
1. Map change scope to tests using `references/test-matrix.md`.
2. Add or update unit tests first (config, parser, aggregator logic).
3. Add integration assertions for `/metrics` and collector enable/disable behavior.
4. Keep privileged tests gated behind `DRISHTI_PRIVILEGED_TESTS=1`.
5. Run `scripts/run_required_tests.sh` before handing off.

## Rules
- Keep tests deterministic and independent from host process state where possible.
- Prefer synthetic event injection for unprivileged integration tests.
- Validate new metric families appear in exposition output.
