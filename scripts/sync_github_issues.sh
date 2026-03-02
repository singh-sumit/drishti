#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage:
  $0 --repo <owner/repo> --input <backlog.yaml> --dry-run
  $0 --repo <owner/repo> --input <backlog.yaml> --apply

Options:
  --repo   GitHub repository slug (required)
  --input  Backlog file (default: internal-docs/issues/backlog.yaml)
  --dry-run  Generate markdown and gh command preview without creating issues
  --apply    Create/update labels, milestones, and issues using gh
USAGE
}

REPO=""
INPUT="internal-docs/issues/backlog.yaml"
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

if [[ "$INPUT" == docs/issues/* ]]; then
  LEGACY_INPUT="$INPUT"
  INPUT="internal-docs/${INPUT#docs/}"
  echo "[warn] legacy path '$LEGACY_INPUT' is deprecated; using '$INPUT'" >&2
fi

python3 - "$REPO" "$INPUT" "$MODE" <<'PY'
import json
import os
import pathlib
import re
import subprocess
import sys
import tempfile
from typing import Any

repo, input_path, mode = sys.argv[1], sys.argv[2], sys.argv[3]

root = pathlib.Path.cwd()
backlog_path = root / input_path
generated_dir = root / "internal-docs/issues/generated"
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


def label_color(label: str) -> str:
    colors = {
        "epic": "5319e7",
        "v0.1": "0052cc",
        "v0.2": "0052cc",
        "v0.3": "0052cc",
        "v0.4": "0052cc",
        "v0.5": "0052cc",
        "drishti": "1f6feb",
        "observability": "0e8a16",
        "deferred": "d4c5f9",
        "testing": "fbca04",
        "ci": "bfd4f2",
        "ebpf": "5319e7",
        "build": "c2e0c6",
        "network": "1d76db",
        "disk": "0e8a16",
        "syscall": "5319e7",
        "qemu": "fbca04",
        "optimization": "c5def5",
    }
    return colors.get(label, "cccccc")


def parse_issue_number(url_or_id: str) -> int:
    match = re.search(r"(\d+)$", url_or_id.strip())
    if not match:
        raise SystemExit(f"unable to parse issue number from: {url_or_id}")
    return int(match.group(1))


def ordered_unique(items: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        if item and item not in seen:
            seen.add(item)
            ordered.append(item)
    return ordered


def ensure_structure(data: dict[str, Any]) -> None:
    if "issues" not in data or not isinstance(data["issues"], list):
        raise SystemExit("backlog must include an 'issues' array")

    ids = [issue.get("id") for issue in data["issues"]]
    if any(not issue_id for issue_id in ids):
        raise SystemExit("every issue must include a non-empty 'id'")
    if len(ids) != len(set(ids)):
        raise SystemExit("issue ids must be unique")

    id_set = set(ids)

    milestones = data.get("milestones", [])
    if milestones and not isinstance(milestones, list):
        raise SystemExit("top-level 'milestones' must be a list when present")

    for issue in data["issues"]:
        deps = issue.get("depends_on", [])
        if not isinstance(deps, list):
            raise SystemExit(f"depends_on for {issue['id']} must be a list")
        unknown_deps = [dep for dep in deps if dep not in id_set]
        if unknown_deps:
            raise SystemExit(f"issue {issue['id']} has unknown dependencies: {unknown_deps}")

        parent_id = issue.get("parent_id")
        if parent_id and parent_id not in id_set:
            raise SystemExit(f"issue {issue['id']} has unknown parent_id: {parent_id}")


def all_milestones(data: dict[str, Any], issues: list[dict[str, Any]]) -> list[str]:
    names: list[str] = []
    names.extend(data.get("milestones", []))
    if data.get("milestone"):
        names.append(data["milestone"])
    for issue in issues:
        milestone = issue.get("milestone")
        if milestone:
            names.append(milestone)
    return ordered_unique(names)


def render_issue_md(issue: dict[str, Any], deps: list[str], parent_id: str | None = None) -> str:
    body = issue.get("body", "").strip()
    lines = [
        f"# {issue['title']}",
        "",
        f"Local ID: `{issue['id']}`",
        f"Type: `{issue.get('type', 'task')}`",
        f"Status: `{issue.get('status', 'planned')}`",
    ]

    milestone = issue.get("milestone")
    if milestone:
        lines.append(f"Milestone: `{milestone}`")

    labels = issue.get("labels", [])
    if labels:
        lines.append(f"Labels: `{', '.join(labels)}`")

    if parent_id:
        lines.append(f"Parent: `{parent_id}`")

    if deps:
        lines.append(f"Depends on: `{', '.join(deps)}`")

    lines.extend(["", body, ""])
    return "\n".join(lines)


def resolve_dependencies(issue: dict[str, Any], issue_map: dict[str, int]) -> list[str]:
    return [f"#{issue_map[dep]}" for dep in issue.get("depends_on", []) if dep in issue_map]


def resolve_children(children_by_parent: dict[str, list[str]], parent_id: str, issue_map: dict[str, int]) -> list[str]:
    refs: list[str] = []
    for child_id in children_by_parent.get(parent_id, []):
        if child_id in issue_map:
            refs.append(f"https://github.com/{repo}/issues/{issue_map[child_id]}")
    return refs


def build_issue_body(
    issue: dict[str, Any],
    issue_map: dict[str, int],
    children_by_parent: dict[str, list[str]],
) -> str:
    base = render_issue_md(issue, issue.get("depends_on", []), issue.get("parent_id"))

    deps = resolve_dependencies(issue, issue_map)
    if deps:
        base += "\n## Dependency Links\n"
        base += "\n".join(f"- {dep}" for dep in deps)
        base += "\n"

    children = resolve_children(children_by_parent, issue["id"], issue_map)
    if children:
        base += "\n## Sub-issues\n"
        base += "\n".join(f"- [ ] {child}" for child in children)
        base += "\n"

    return base


def write_issue_markdowns(issues: list[dict[str, Any]]) -> None:
    for issue in issues:
        deps = issue.get("depends_on", [])
        md = render_issue_md(issue, deps, issue.get("parent_id"))
        (generated_dir / f"{sanitize_filename(issue['id'])}.md").write_text(md, encoding="utf-8")


def generate_command_preview(
    data: dict[str, Any],
    issues: list[dict[str, Any]],
    issue_map: dict[str, int],
) -> None:
    milestone_names = all_milestones(data, issues)
    all_labels = ordered_unique(
        [*(data.get("labels", [])), *[label for issue in issues for label in issue.get("labels", [])]]
    )

    lines = ["#!/usr/bin/env bash", "set -euo pipefail", ""]

    for milestone in milestone_names:
        lines.append(f"gh api repos/{repo}/milestones -f title='{milestone}' || true")

    for label in all_labels:
        lines.append(f"gh label create --repo {repo} '{label}' --color {label_color(label)} --force")

    for issue in issues:
        body_file = f"internal-docs/issues/generated/{sanitize_filename(issue['id'])}.md"
        local_id = issue["id"]
        labels = issue.get("labels", [])
        milestone = issue.get("milestone")

        if local_id in issue_map:
            line = f"gh issue edit --repo {repo} {issue_map[local_id]} --title '{issue['title']}' --body-file '{body_file}'"
            if milestone:
                line += f" --milestone '{milestone}'"
            lines.append(line)
        else:
            label_flags = " ".join([f"--label '{label}'" for label in labels])
            milestone_flag = f" --milestone '{milestone}'" if milestone else ""
            lines.append(
                (
                    f"gh issue create --repo {repo} --title '{issue['title']}' --body-file '{body_file}' "
                    f"{label_flags}{milestone_flag}"
                ).strip()
            )

    lines.append("# parent tasklists and dependency links are applied during --apply")

    for issue in issues:
        if issue.get("status", "planned") == "done":
            lines.append(
                f"# close when mapped: {issue['id']} -> gh issue close --repo {repo} <issue-number>"
            )

    commands_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    os.chmod(commands_path, 0o755)


def ensure_milestones(repo: str, milestone_names: list[str]) -> None:
    if not milestone_names:
        return

    existing_raw = run_cmd(["gh", "api", f"repos/{repo}/milestones?state=all&per_page=100"], check=True)
    existing = json.loads(existing_raw or "[]")
    existing_titles = {milestone["title"] for milestone in existing}

    for milestone in milestone_names:
        if milestone in existing_titles:
            continue
        run_cmd(["gh", "api", f"repos/{repo}/milestones", "-f", f"title={milestone}"], check=True)


def sync_labels(repo: str, labels: list[str]) -> None:
    for label in labels:
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


def ensure_issue_exists(
    repo: str,
    issue: dict[str, Any],
    issue_map: dict[str, int],
) -> None:
    local_id = issue["id"]
    if local_id in issue_map:
        return

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

    milestone = issue.get("milestone")
    if milestone:
        cmd.extend(["--milestone", milestone])

    out = run_cmd(cmd, check=True)
    issue_map[local_id] = parse_issue_number(out)


def sync_issue_details(
    repo: str,
    issue: dict[str, Any],
    issue_number: int,
    desired_body: str,
) -> None:
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as temp_file:
        temp_file.write(desired_body)
        temp_path = pathlib.Path(temp_file.name)

    try:
        cmd = [
            "gh",
            "issue",
            "edit",
            "--repo",
            repo,
            str(issue_number),
            "--title",
            issue["title"],
            "--body-file",
            str(temp_path),
        ]
        milestone = issue.get("milestone")
        if milestone:
            cmd.extend(["--milestone", milestone])
        run_cmd(cmd, check=True)

        current_raw = run_cmd(
            ["gh", "issue", "view", "--repo", repo, str(issue_number), "--json", "labels"],
            check=True,
        )
        current = json.loads(current_raw)
        current_labels = {label["name"] for label in current.get("labels", [])}
        desired_labels = set(issue.get("labels", []))

        add_labels = sorted(desired_labels - current_labels)
        remove_labels = sorted(current_labels - desired_labels)

        if add_labels or remove_labels:
            label_cmd = ["gh", "issue", "edit", "--repo", repo, str(issue_number)]
            for label in add_labels:
                label_cmd.extend(["--add-label", label])
            for label in remove_labels:
                label_cmd.extend(["--remove-label", label])
            run_cmd(label_cmd, check=True)
    finally:
        temp_path.unlink(missing_ok=True)


def sync_issue_state(repo: str, issue: dict[str, Any], issue_number: int) -> None:
    state_raw = run_cmd(
        ["gh", "issue", "view", "--repo", repo, str(issue_number), "--json", "state"],
        check=True,
    )
    state = json.loads(state_raw).get("state", "OPEN")
    desired_status = issue.get("status", "planned")

    if desired_status == "done" and state == "OPEN":
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
            check=True,
        )

    if desired_status != "done" and state == "CLOSED":
        run_cmd(["gh", "issue", "reopen", "--repo", repo, str(issue_number)], check=True)


data = load_backlog(backlog_path)
ensure_structure(data)
generated_dir.mkdir(parents=True, exist_ok=True)

issues: list[dict[str, Any]] = data["issues"]
write_issue_markdowns(issues)

if map_path.exists():
    issue_map: dict[str, int] = {
        key: int(value)
        for key, value in json.loads(map_path.read_text(encoding="utf-8")).items()
    }
else:
    issue_map = {}

generate_command_preview(data, issues, issue_map)

if mode == "dry-run":
    print(f"[dry-run] generated markdown in {generated_dir}")
    print(f"[dry-run] command preview written to {commands_path}")
    print(f"[dry-run] issues in sync order: {', '.join(issue['id'] for issue in issues)}")
    raise SystemExit(0)

run_cmd(["gh", "--version"], check=True)
run_cmd(["gh", "auth", "status"], check=True)

milestone_names = all_milestones(data, issues)
all_labels = ordered_unique(
    [*(data.get("labels", [])), *[label for issue in issues for label in issue.get("labels", [])]]
)

ensure_milestones(repo, milestone_names)
sync_labels(repo, all_labels)

for issue in issues:
    ensure_issue_exists(repo, issue, issue_map)

children_by_parent: dict[str, list[str]] = {}
for issue in issues:
    parent_id = issue.get("parent_id")
    if not parent_id:
        continue
    children_by_parent.setdefault(parent_id, []).append(issue["id"])

for issue in issues:
    local_id = issue["id"]
    issue_number = issue_map.get(local_id)
    if issue_number is None:
        raise SystemExit(f"missing issue number mapping for {local_id}")

    body = build_issue_body(issue, issue_map, children_by_parent)
    body_file = generated_dir / f"{sanitize_filename(local_id)}.md"
    body_file.write_text(body, encoding="utf-8")

    sync_issue_details(repo, issue, issue_number, body)
    sync_issue_state(repo, issue, issue_number)

map_path.write_text(json.dumps(issue_map, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"[apply] synced {len(issues)} issues to {repo}")
print(f"[apply] issue mapping stored at {map_path}")
PY
