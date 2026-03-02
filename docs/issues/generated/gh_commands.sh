#!/usr/bin/env bash
set -euo pipefail

gh api repos/singh-sumit/drishti/milestones -f title='v0.1 Drishti Core' || true
gh api repos/singh-sumit/drishti/milestones -f title='v0.2 Drishti Network+Disk' || true
gh api repos/singh-sumit/drishti/milestones -f title='v0.3 Drishti Syscalls' || true
gh api repos/singh-sumit/drishti/milestones -f title='v0.4 Drishti QEMU CI' || true
gh api repos/singh-sumit/drishti/milestones -f title='v0.5 Drishti Packaging' || true
gh label create --repo singh-sumit/drishti 'drishti' --color 1f6feb --force
gh label create --repo singh-sumit/drishti 'observability' --color 0e8a16 --force
gh label create --repo singh-sumit/drishti 'v0.1' --color 0052cc --force
gh label create --repo singh-sumit/drishti 'v0.2' --color 0052cc --force
gh label create --repo singh-sumit/drishti 'v0.3' --color 0052cc --force
gh label create --repo singh-sumit/drishti 'v0.4' --color 0052cc --force
gh label create --repo singh-sumit/drishti 'v0.5' --color 0052cc --force
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
gh label create --repo singh-sumit/drishti 'network' --color 1d76db --force
gh label create --repo singh-sumit/drishti 'docs' --color cccccc --force
gh label create --repo singh-sumit/drishti 'disk' --color 0e8a16 --force
gh label create --repo singh-sumit/drishti 'syscall' --color 5319e7 --force
gh label create --repo singh-sumit/drishti 'qemu' --color fbca04 --force
gh label create --repo singh-sumit/drishti 'optimization' --color c5def5 --force
gh issue edit --repo singh-sumit/drishti 1 --title 'Epic: Drishti v0.1 Core Delivery' --body-file 'docs/issues/generated/DRISHTI-001.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 2 --title 'Bootstrap Rust workspace and build automation' --body-file 'docs/issues/generated/DRISHTI-010.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 3 --title 'Implement shared ABI-safe event and map types' --body-file 'docs/issues/generated/DRISHTI-020.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 4 --title 'Implement eBPF scheduler and process probes' --body-file 'docs/issues/generated/DRISHTI-030.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 5 --title 'Implement daemon loader and collector pipeline' --body-file 'docs/issues/generated/DRISHTI-040.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 6 --title 'Implement memory procfs collector and filters' --body-file 'docs/issues/generated/DRISHTI-050.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 7 --title 'Add Prometheus exporter and metric aggregation guards' --body-file 'docs/issues/generated/DRISHTI-060.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 8 --title 'Provision Grafana dashboards and deployment files' --body-file 'docs/issues/generated/DRISHTI-070.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 9 --title 'Implement CI workflows and gated privileged smoke tests' --body-file 'docs/issues/generated/DRISHTI-080.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 10 --title 'Deferred: Network telemetry collector' --body-file 'docs/issues/generated/DRISHTI-090.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Network: ABI types and config surface' --body-file 'docs/issues/generated/DRISHTI-100.md' --label 'network' --label 'rust' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Network: eBPF probe and ring buffer events' --body-file 'docs/issues/generated/DRISHTI-101.md' --label 'network' --label 'ebpf' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Network: daemon aggregation and exporter metrics' --body-file 'docs/issues/generated/DRISHTI-102.md' --label 'network' --label 'daemon' --label 'metrics' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Network: tests and privileged smoke coverage' --body-file 'docs/issues/generated/DRISHTI-103.md' --label 'network' --label 'testing' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Network: docs and dashboard updates' --body-file 'docs/issues/generated/DRISHTI-104.md' --label 'network' --label 'grafana' --label 'docs' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 11 --title 'Deferred: Disk I/O collector' --body-file 'docs/issues/generated/DRISHTI-091.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Disk: ABI types and config surface' --body-file 'docs/issues/generated/DRISHTI-110.md' --label 'disk' --label 'rust' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Disk: eBPF block I/O probes' --body-file 'docs/issues/generated/DRISHTI-111.md' --label 'disk' --label 'ebpf' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Disk: daemon aggregation and exporter metrics' --body-file 'docs/issues/generated/DRISHTI-112.md' --label 'disk' --label 'daemon' --label 'metrics' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Disk: tests and privileged smoke coverage' --body-file 'docs/issues/generated/DRISHTI-113.md' --label 'disk' --label 'testing' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue create --repo singh-sumit/drishti --title 'Disk: docs and dashboard updates' --body-file 'docs/issues/generated/DRISHTI-114.md' --label 'disk' --label 'grafana' --label 'docs' --label 'v0.2' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 12 --title 'Deferred: Syscall tracing collector' --body-file 'docs/issues/generated/DRISHTI-092.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 13 --title 'Deferred: Cross-arch QEMU CI execution' --body-file 'docs/issues/generated/DRISHTI-093.md' --milestone 'v0.4 Drishti QEMU CI'
gh issue edit --repo singh-sumit/drishti 14 --title 'Deferred: musl static optimization and size budgets' --body-file 'docs/issues/generated/DRISHTI-094.md' --milestone 'v0.5 Drishti Packaging'
# parent tasklists and dependency links are applied during --apply
# close when mapped: DRISHTI-001 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-010 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-020 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-030 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-040 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-050 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-060 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-070 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-080 -> gh issue close --repo singh-sumit/drishti <issue-number>
