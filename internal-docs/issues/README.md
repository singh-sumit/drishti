# Drishti Issue Backlog

Use `internal-docs/issues/backlog.yaml` as the source of truth for local planning.

## Backlog schema

Top-level keys:
- `milestones`: milestone names to ensure in GitHub (created on sync)
- `labels`: common labels to ensure in GitHub
- `issues`: ordered issue definitions

Issue-level keys:
- `id` (required): local issue id (e.g. `DRISHTI-090`)
- `title` (required)
- `type`: `task` or `epic`
- `labels`: list of GitHub labels
- `milestone`: milestone name
- `status`: `planned`, `in-progress`, `done`
- `depends_on`: local ids this issue depends on
- `parent_id`: optional local id of parent issue (for deterministic sub-issue tasklists)
- `body`: markdown body

`status` handling during sync:
- `done`: closes mapped GitHub issue on `--apply`
- `planned` or `in-progress`: keeps issue open

Regenerate issue markdown and command previews:

```bash
scripts/sync_github_issues.sh --repo <owner/repo> --input internal-docs/issues/backlog.yaml --dry-run
```

Apply to GitHub (requires `gh auth` and network access):

```bash
scripts/sync_github_issues.sh --repo <owner/repo> --input internal-docs/issues/backlog.yaml --apply
```
