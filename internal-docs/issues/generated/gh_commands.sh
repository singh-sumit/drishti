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
gh label create --repo singh-sumit/drishti 'docusaurus' --color cccccc --force
gh label create --repo singh-sumit/drishti 'migration' --color cccccc --force
gh label create --repo singh-sumit/drishti 'architecture' --color cccccc --force
gh label create --repo singh-sumit/drishti 'operations' --color cccccc --force
gh label create --repo singh-sumit/drishti 'integrations' --color cccccc --force
gh label create --repo singh-sumit/drishti 'github-pages' --color cccccc --force
gh label create --repo singh-sumit/drishti 'developer-experience' --color cccccc --force
gh issue edit --repo singh-sumit/drishti 1 --title 'Epic: Drishti v0.1 Core Delivery' --body-file 'internal-docs/issues/generated/DRISHTI-001.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 2 --title 'Bootstrap Rust workspace and build automation' --body-file 'internal-docs/issues/generated/DRISHTI-010.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 3 --title 'Implement shared ABI-safe event and map types' --body-file 'internal-docs/issues/generated/DRISHTI-020.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 4 --title 'Implement eBPF scheduler and process probes' --body-file 'internal-docs/issues/generated/DRISHTI-030.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 5 --title 'Implement daemon loader and collector pipeline' --body-file 'internal-docs/issues/generated/DRISHTI-040.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 6 --title 'Implement memory procfs collector and filters' --body-file 'internal-docs/issues/generated/DRISHTI-050.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 7 --title 'Add Prometheus exporter and metric aggregation guards' --body-file 'internal-docs/issues/generated/DRISHTI-060.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 8 --title 'Provision Grafana dashboards and deployment files' --body-file 'internal-docs/issues/generated/DRISHTI-070.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 9 --title 'Implement CI workflows and gated privileged smoke tests' --body-file 'internal-docs/issues/generated/DRISHTI-080.md' --milestone 'v0.1 Drishti Core'
gh issue edit --repo singh-sumit/drishti 10 --title 'Deferred: Network telemetry collector' --body-file 'internal-docs/issues/generated/DRISHTI-090.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 16 --title 'Network: ABI types and config surface' --body-file 'internal-docs/issues/generated/DRISHTI-100.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 17 --title 'Network: eBPF probe and ring buffer events' --body-file 'internal-docs/issues/generated/DRISHTI-101.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 18 --title 'Network: daemon aggregation and exporter metrics' --body-file 'internal-docs/issues/generated/DRISHTI-102.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 19 --title 'Network: tests and privileged smoke coverage' --body-file 'internal-docs/issues/generated/DRISHTI-103.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 20 --title 'Network: docs and dashboard updates' --body-file 'internal-docs/issues/generated/DRISHTI-104.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 11 --title 'Deferred: Disk I/O collector' --body-file 'internal-docs/issues/generated/DRISHTI-091.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 21 --title 'Disk: ABI types and config surface' --body-file 'internal-docs/issues/generated/DRISHTI-110.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 22 --title 'Disk: eBPF block I/O probes' --body-file 'internal-docs/issues/generated/DRISHTI-111.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 23 --title 'Disk: daemon aggregation and exporter metrics' --body-file 'internal-docs/issues/generated/DRISHTI-112.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 24 --title 'Disk: tests and privileged smoke coverage' --body-file 'internal-docs/issues/generated/DRISHTI-113.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 25 --title 'Disk: docs and dashboard updates' --body-file 'internal-docs/issues/generated/DRISHTI-114.md' --milestone 'v0.2 Drishti Network+Disk'
gh issue edit --repo singh-sumit/drishti 12 --title 'Deferred: Syscall tracing collector' --body-file 'internal-docs/issues/generated/DRISHTI-092.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 27 --title 'Syscall: ABI types and config surface' --body-file 'internal-docs/issues/generated/DRISHTI-120.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 28 --title 'Syscall: eBPF enter/exit probes and maps' --body-file 'internal-docs/issues/generated/DRISHTI-121.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 29 --title 'Syscall: daemon collector and metrics aggregation' --body-file 'internal-docs/issues/generated/DRISHTI-122.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 30 --title 'Syscall: tests and privileged smoke coverage' --body-file 'internal-docs/issues/generated/DRISHTI-123.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 31 --title 'Syscall: dashboards and docs updates' --body-file 'internal-docs/issues/generated/DRISHTI-124.md' --milestone 'v0.3 Drishti Syscalls'
gh issue edit --repo singh-sumit/drishti 13 --title 'Deferred: Cross-arch QEMU CI execution' --body-file 'internal-docs/issues/generated/DRISHTI-093.md' --milestone 'v0.4 Drishti QEMU CI'
gh issue edit --repo singh-sumit/drishti 14 --title 'Deferred: musl static optimization and size budgets' --body-file 'internal-docs/issues/generated/DRISHTI-094.md' --milestone 'v0.5 Drishti Packaging'
gh issue create --repo singh-sumit/drishti --title 'Docs Portal: Drishti Engineering Documentation' --body-file 'internal-docs/issues/generated/DRISHTI-130.md' --label 'docs' --label 'docusaurus' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: full docs/ -> internal-docs/ migration' --body-file 'internal-docs/issues/generated/DRISHTI-131.md' --label 'docs' --label 'migration' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: Docusaurus bootstrap + Mermaid integration' --body-file 'internal-docs/issues/generated/DRISHTI-132.md' --label 'docs' --label 'docusaurus' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: architecture and data-flow content from spec' --body-file 'internal-docs/issues/generated/DRISHTI-133.md' --label 'docs' --label 'architecture' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: usage, operations, troubleshooting content' --body-file 'internal-docs/issues/generated/DRISHTI-134.md' --label 'docs' --label 'operations' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: integration guides (Prometheus/Grafana/systemd)' --body-file 'internal-docs/issues/generated/DRISHTI-135.md' --label 'docs' --label 'integrations' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: GitHub Pages workflow and release checks' --body-file 'internal-docs/issues/generated/DRISHTI-136.md' --label 'docs' --label 'ci' --label 'github-pages' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: README + developer workflow + rustdoc reference linking' --body-file 'internal-docs/issues/generated/DRISHTI-137.md' --label 'docs' --label 'developer-experience' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
gh issue create --repo singh-sumit/drishti --title 'Docs: closeout, validation evidence, issue sync finalization' --body-file 'internal-docs/issues/generated/DRISHTI-138.md' --label 'docs' --label 'testing' --label 'v0.4' --milestone 'v0.4 Drishti QEMU CI'
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
# close when mapped: DRISHTI-090 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-100 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-101 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-102 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-103 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-104 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-091 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-110 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-111 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-112 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-113 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-114 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-092 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-120 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-121 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-122 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-123 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-124 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-130 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-131 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-132 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-133 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-134 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-135 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-136 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-137 -> gh issue close --repo singh-sumit/drishti <issue-number>
# close when mapped: DRISHTI-138 -> gh issue close --repo singh-sumit/drishti <issue-number>
