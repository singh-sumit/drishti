# Drishti Issue Backlog

Use `docs/issues/backlog.yaml` as the source of truth for local planning.

`status` handling during sync:
- `done`: closes mapped GitHub issue on `--apply`
- `planned` or `in-progress`: keeps issue open

Regenerate issue markdown and command previews:

```bash
scripts/sync_github_issues.sh --repo <owner/repo> --input docs/issues/backlog.yaml --dry-run
```

Apply to GitHub (requires `gh auth` and network access):

```bash
scripts/sync_github_issues.sh --repo <owner/repo> --input docs/issues/backlog.yaml --apply
```
