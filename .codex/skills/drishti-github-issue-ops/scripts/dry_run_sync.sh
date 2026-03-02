#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$ROOT_DIR"

scripts/sync_github_issues.sh --repo "${1:-owner/repo}" --input docs/issues/backlog.yaml --dry-run
