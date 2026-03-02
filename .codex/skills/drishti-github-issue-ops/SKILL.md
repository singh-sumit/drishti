---
name: drishti-github-issue-ops
description: Operate the Drishti local-first GitHub issue workflow. Use when creating/updating backlog entries in docs/issues, regenerating issue markdown, and syncing issues/labels/milestones to GitHub with gh.
---

# Drishti GitHub Issue Ops

## Workflow
1. Update `docs/issues/backlog.yaml` using the schema in `references/backlog-schema.md`.
2. Regenerate issue markdown and command previews by running `scripts/sync_github_issues.sh --dry-run`.
3. Review generated files in `docs/issues/generated/`.
4. Apply to GitHub with `scripts/sync_github_issues.sh --apply` when auth/network are ready.

## Rules
- Keep issue IDs stable (`DRISHTI-###`).
- Keep dependencies explicit and acyclic.
- Keep creation order deterministic (epic before dependent tasks).
