#!/usr/bin/env bash
set -euo pipefail

gh api repos/singh-sumit/drishti/milestones -f title='v0.1 Drishti Core' || true
gh label create --repo singh-sumit/drishti 'drishti' --color 1f6feb --force
gh label create --repo singh-sumit/drishti 'observability' --color 0e8a16 --force
gh label create --repo singh-sumit/drishti 'v0.1' --color 0052cc --force
gh label create --repo singh-sumit/drishti 'epic' --color 5319e7 --force
gh label create --repo singh-sumit/drishti 'build' --color c2e0c6 --force
gh label create --repo singh-sumit/drishti 'rust' --color cccccc --force
gh label create --repo singh-sumit/drishti 'ebpf' --color 5319e7 --force
gh label create --repo singh-sumit/drishti 'cpu' --color cccccc --force
gh label create --repo singh-sumit/drishti 'process' --color cccccc --force
gh label create --repo singh-sumit/drishti 'daemon' --color cccccc --force
gh label create --repo singh-sumit/drishti 'collector' --color cccccc --force
gh label create --repo singh-sumit/drishti 'memory' --color cccccc --force
gh label create --repo singh-sumit/drishti 'procfs' --color cccccc --force
gh label create --repo singh-sumit/drishti 'metrics' --color cccccc --force
gh label create --repo singh-sumit/drishti 'prometheus' --color cccccc --force
gh label create --repo singh-sumit/drishti 'grafana' --color cccccc --force
gh label create --repo singh-sumit/drishti 'deploy' --color cccccc --force
gh label create --repo singh-sumit/drishti 'ci' --color bfd4f2 --force
gh label create --repo singh-sumit/drishti 'testing' --color fbca04 --force
gh label create --repo singh-sumit/drishti 'deferred' --color d4c5f9 --force
gh label create --repo singh-sumit/drishti 'network' --color cccccc --force
gh label create --repo singh-sumit/drishti 'disk' --color cccccc --force
gh label create --repo singh-sumit/drishti 'syscall' --color cccccc --force
gh label create --repo singh-sumit/drishti 'qemu' --color cccccc --force
gh label create --repo singh-sumit/drishti 'optimization' --color cccccc --force
gh issue create --repo singh-sumit/drishti --title 'Epic: Drishti v0.1 Core Delivery' --body-file 'docs/issues/generated/DRISHTI-001.md' --label 'epic' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Bootstrap Rust workspace and build automation' --body-file 'docs/issues/generated/DRISHTI-010.md' --label 'build' --label 'rust' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Implement shared ABI-safe event and map types' --body-file 'docs/issues/generated/DRISHTI-020.md' --label 'rust' --label 'ebpf' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Implement eBPF scheduler and process probes' --body-file 'docs/issues/generated/DRISHTI-030.md' --label 'ebpf' --label 'cpu' --label 'process' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Implement daemon loader and collector pipeline' --body-file 'docs/issues/generated/DRISHTI-040.md' --label 'daemon' --label 'collector' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Implement memory procfs collector and filters' --body-file 'docs/issues/generated/DRISHTI-050.md' --label 'memory' --label 'procfs' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Add Prometheus exporter and metric aggregation guards' --body-file 'docs/issues/generated/DRISHTI-060.md' --label 'metrics' --label 'prometheus' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Provision Grafana dashboards and deployment files' --body-file 'docs/issues/generated/DRISHTI-070.md' --label 'grafana' --label 'deploy' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Implement CI workflows and gated privileged smoke tests' --body-file 'docs/issues/generated/DRISHTI-080.md' --label 'ci' --label 'testing' --label 'v0.1' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Deferred: Network telemetry collector' --body-file 'docs/issues/generated/DRISHTI-090.md' --label 'deferred' --label 'network' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Deferred: Disk I/O collector' --body-file 'docs/issues/generated/DRISHTI-091.md' --label 'deferred' --label 'disk' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Deferred: Syscall tracing collector' --body-file 'docs/issues/generated/DRISHTI-092.md' --label 'deferred' --label 'syscall' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Deferred: Cross-arch QEMU CI execution' --body-file 'docs/issues/generated/DRISHTI-093.md' --label 'deferred' --label 'qemu' --label 'ci' --milestone 'v0.1 Drishti Core'
gh issue create --repo singh-sumit/drishti --title 'Deferred: musl static optimization and size budgets' --body-file 'docs/issues/generated/DRISHTI-094.md' --label 'deferred' --label 'build' --label 'optimization' --milestone 'v0.1 Drishti Core'
# close when mapped: DRISHTI-001 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-010 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-020 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-030 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-040 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-050 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-060 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-070 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
# close when mapped: DRISHTI-080 -> gh issue close --repo singh-sumit/drishti <issue-number> --comment 'Closing from local backlog sync: status=done'
