---
title: Performance Budgets
sidebar_position: 2
---

The design target from the architecture spec emphasizes low overhead on constrained systems.

## Practical Budgets

- keep daemon idle RSS low and stable
- keep collector CPU overhead bounded under sustained event load
- avoid unbounded metric series growth

## Tuning Levers

1. disable high-volume collectors not needed for a host role
2. adjust scrape and polling intervals
3. constrain interfaces/devices lists
4. tune syscall `top_n` and disable syscall collector where unnecessary

## Guardrails in Code

- `max_series` circuit breaker
- bounded map sizes in eBPF programs
- non-fatal attach strategy with explicit logs
