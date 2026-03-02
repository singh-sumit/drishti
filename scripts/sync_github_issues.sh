#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage:
  $0 --repo <owner/repo> --input <backlog.yaml> --dry-run
  $0 --repo <owner/repo> --input <backlog.yaml> --apply

Options:
  --repo   GitHub repository slug (required)
  --input  Backlog file (default: docs/issues/backlog.yaml)
  --dry-run  Generate markdown and gh command preview without creating issues
  --apply    Create labels/milestone/issues and dependency comments via gh
USAGE
}

REPO=""
INPUT="docs/issues/backlog.yaml"
MODE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --input)
      INPUT="${2:-}"
      shift 2
      ;;
    --dry-run)
      MODE="dry-run"
      shift
      ;;
    --apply)
      MODE="apply"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$REPO" || -z "$MODE" ]]; then
  usage
  exit 1
fi

python3 - "$REPO" "$INPUT" "$MODE" <<'PY'
import argparse
import json
import os
import pathlib
import re
import subprocess
import sys
from typing import Any

repo, input_path, mode = sys.argv[1], sys.argv[2], sys.argv[3]

root = pathlib.Path.cwd()
backlog_path = root / input_path
generated_dir = root / "docs/issues/generated"
commands_path = generated_dir / "gh_commands.sh"
map_path = generated_dir / "issue_ids.json"

def load_backlog(path: pathlib.Path) -> dict[str, Any]:
    raw = path.read_text(encoding="utf-8")
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        try:
            import yaml  # type: ignore
        except ModuleNotFoundError as exc:
            raise SystemExit(
                "backlog is not JSON-compatible YAML and PyYAML is unavailable"
            ) from exc
        return yaml.safe_load(raw)


def run_cmd(args: list[str], check: bool = True) -> str:
    proc = subprocess.run(args, capture_output=True, text=True)
    if check and proc.returncode != 0:
        raise SystemExit(f"command failed: {' '.join(args)}\n{proc.stderr.strip()}")
    return proc.stdout.strip()


def sanitize_filename(issue_id: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]", "_", issue_id)


def ensure_structure(data: dict[str, Any]) -> None:
    if "issues" not in data or not isinstance(data["issues"], list):
        raise SystemExit("backlog must include an 'issues' array")
    ids = [issue.get("id") for issue in data["issues"]]
    if len(ids) != len(set(ids)):
        raise SystemExit("issue ids must be unique")


def render_issue_md(issue: dict[str, Any], deps: list[str]) -> str:
    body = issue.get("body", "").strip()
    lines = [
        f"# {issue['title']}",
        "",
        f"Local ID: `{issue['id']}`",
        f"Type: `{issue.get('type', 'task')}`",
        f"Status: `{issue.get('status', 'planned')}`",
    ]
    labels = issue.get("labels", [])
    if labels:
        lines.append(f"Labels: `{', '.join(labels)}`")
    if deps:
        lines.append(f"Depends on: `{', '.join(deps)}`")
    lines.extend(["", body, ""])
    return "\n".join(lines)


def label_color(label: str) -> str:
    colors = {
        "epic": "5319e7",
        "v0.1": "0052cc",
        "drishti": "1f6feb",
        "observability": "0e8a16",
        "deferred": "d4c5f9",
        "testing": "fbca04",
        "ci": "bfd4f2",
        "ebpf": "5319e7",
        "build": "c2e0c6",
    }
    return colors.get(label, "cccccc")


def parse_issue_number(url_or_id: str) -> int:
    match = re.search(r"(\d+)$", url_or_id.strip())
    if not match:
        raise SystemExit(f"unable to parse issue number from: {url_or_id}")
    return int(match.group(1))


data = load_backlog(backlog_path)
ensure_structure(data)
generated_dir.mkdir(parents=True, exist_ok=True)

issues: list[dict[str, Any]] = data["issues"]
all_labels = list(dict.fromkeys([*(data.get("labels", [])), *[l for i in issues for l in i.get("labels", [])]]))
milestone = data.get("milestone", "")

for issue in issues:
    deps = issue.get("depends_on", [])
    md = render_issue_md(issue, deps)
    (generated_dir / f"{sanitize_filename(issue['id'])}.md").write_text(md, encoding="utf-8")

lines = ["#!/usr/bin/env bash", "set -euo pipefail", ""]
if milestone:
    lines.append(f"gh api repos/{repo}/milestones -f title='{milestone}' || true")
for label in all_labels:
    lines.append(
        f"gh label create --repo {repo} '{label}' --color {label_color(label)} --force"
    )
for issue in issues:
    body_file = f"docs/issues/generated/{sanitize_filename(issue['id'])}.md"
    label_flags = " ".join([f"--label '{label}'" for label in issue.get("labels", [])])
    milestone_flag = f"--milestone '{milestone}'" if milestone else ""
    lines.append(
        f"gh issue create --repo {repo} --title '{issue['title']}' --body-file '{body_file}' {label_flags} {milestone_flag}".strip()
    )
for issue in issues:
    if issue.get("status", "planned") == "done":
        lines.append(
            f"# close when mapped: {issue['id']} -> gh issue close --repo {repo} <issue-number> --comment 'Closing from local backlog sync: status=done'"
        )

commands_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
os.chmod(commands_path, 0o755)

if mode == "dry-run":
    print(f"[dry-run] generated markdown in {generated_dir}")
    print(f"[dry-run] command preview written to {commands_path}")
    print(f"[dry-run] issues in creation order: {', '.join(issue['id'] for issue in issues)}")
    raise SystemExit(0)

run_cmd(["gh", "--version"], check=True)
run_cmd(["gh", "auth", "status"], check=True)

if map_path.exists():
    issue_map = json.loads(map_path.read_text(encoding="utf-8"))
else:
    issue_map = {}

if milestone:
    run_cmd(["gh", "api", f"repos/{repo}/milestones", "-f", f"title={milestone}"], check=False)

for label in all_labels:
    run_cmd(
        [
            "gh",
            "label",
            "create",
            "--repo",
            repo,
            label,
            "--color",
            label_color(label),
            "--force",
        ],
        check=True,
    )

for issue in issues:
    local_id = issue["id"]
    if local_id in issue_map:
        continue

    body_file = generated_dir / f"{sanitize_filename(local_id)}.md"
    cmd = [
        "gh",
        "issue",
        "create",
        "--repo",
        repo,
        "--title",
        issue["title"],
        "--body-file",
        str(body_file),
    ]
    for label in issue.get("labels", []):
        cmd.extend(["--label", label])
    if milestone:
        cmd.extend(["--milestone", milestone])

    out = run_cmd(cmd, check=True)
    issue_map[local_id] = parse_issue_number(out)

for issue in issues:
    deps = issue.get("depends_on", [])
    if not deps:
        continue
    local_id = issue["id"]
    issue_number = issue_map.get(local_id)
    resolved = [f"#{issue_map[dep]}" for dep in deps if dep in issue_map]
    if not issue_number or not resolved:
        continue

    comment = "Depends on: " + ", ".join(resolved)
    run_cmd(
        [
            "gh",
            "issue",
            "comment",
            "--repo",
            repo,
            str(issue_number),
            "--body",
            comment,
        ],
        check=True,
    )

for issue in issues:
    if issue.get("status", "planned") != "done":
        continue
    local_id = issue["id"]
    issue_number = issue_map.get(local_id)
    if not issue_number:
        continue
    run_cmd(
        [
            "gh",
            "issue",
            "close",
            "--repo",
            repo,
            str(issue_number),
            "--comment",
            "Closing from local backlog sync: status=done",
        ],
        check=False,
    )

map_path.write_text(json.dumps(issue_map, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"[apply] synced {len(issues)} issues to {repo}")
print(f"[apply] issue mapping stored at {map_path}")
PY
