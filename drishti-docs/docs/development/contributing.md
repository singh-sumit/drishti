---
title: Contributing
sidebar_position: 1
---

## Local Development Loop

```bash
just fmt-check
just lint
just test
```

## Docs Development Loop

```bash
just docs-install
just docs-dev
```

Build docs for CI-equivalent checks:

```bash
just docs-verify
```

## Branch and PR Expectations

1. keep changes scoped per milestone/issue.
2. update tests with behavior changes.
3. update docs when config, metrics, or deployment behavior changes.
