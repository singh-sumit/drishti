# Backlog Schema

`docs/issues/backlog.yaml` is JSON-compatible YAML with top-level keys:
- `milestone`: string
- `labels`: array of label strings
- `issues`: ordered array of objects

Issue object fields:
- `id`: stable local identifier (`DRISHTI-###`)
- `title`: issue title
- `type`: `epic` or `task`
- `labels`: array of labels
- `body`: markdown text
- `depends_on`: array of local IDs
- `status`: `planned` | `in-progress` | `done`

The sync script resolves `depends_on` to GitHub issue references when known.
