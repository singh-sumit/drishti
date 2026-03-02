#!/usr/bin/env python3
"""Validate minimal structure for repo-local Codex skills without external deps."""

from __future__ import annotations

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SKILLS_ROOT = ROOT / ".codex" / "skills"


def fail(message: str) -> None:
    print(f"[error] {message}")
    raise SystemExit(1)


if not SKILLS_ROOT.exists():
    fail(f"missing skills root: {SKILLS_ROOT}")

skill_dirs = sorted(path for path in SKILLS_ROOT.iterdir() if path.is_dir())
if not skill_dirs:
    fail("no skills found")

for skill_dir in skill_dirs:
    skill_md = skill_dir / "SKILL.md"
    openai_yaml = skill_dir / "agents" / "openai.yaml"

    if not skill_md.exists():
        fail(f"{skill_dir.name}: missing SKILL.md")
    if not openai_yaml.exists():
        fail(f"{skill_dir.name}: missing agents/openai.yaml")

    raw = skill_md.read_text(encoding="utf-8")
    if not raw.startswith("---\n"):
        fail(f"{skill_dir.name}: SKILL.md missing YAML frontmatter")

    lines = raw.splitlines()
    frontmatter_end = None
    for idx in range(1, len(lines)):
        if lines[idx].strip() == "---":
            frontmatter_end = idx
            break
    if frontmatter_end is None:
        fail(f"{skill_dir.name}: frontmatter not terminated")

    frontmatter = lines[1:frontmatter_end]
    if not any(line.startswith("name:") for line in frontmatter):
        fail(f"{skill_dir.name}: frontmatter missing name")
    if not any(line.startswith("description:") for line in frontmatter):
        fail(f"{skill_dir.name}: frontmatter missing description")

    ui = openai_yaml.read_text(encoding="utf-8")
    required_snippets = ["display_name:", "short_description:", "default_prompt:"]
    for snippet in required_snippets:
        if snippet not in ui:
            fail(f"{skill_dir.name}: openai.yaml missing {snippet}")

print(f"[ok] validated {len(skill_dirs)} skills")
