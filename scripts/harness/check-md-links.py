#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")

SKIP_DIRS = {
    ".code-review-graph",
    ".git",
    ".serena",
    "target",
}


def iter_markdown_files() -> list[Path]:
    files: list[Path] = []
    for path in ROOT.rglob("*.md"):
        if any(part in SKIP_DIRS for part in path.relative_to(ROOT).parts):
            continue
        files.append(path)
    return sorted(files)


def is_external(target: str) -> bool:
    return (
        "://" in target
        or target.startswith("mailto:")
        or target.startswith("#")
        or target.startswith("app://")
        or target.startswith("plugin://")
    )


def clean_target(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1]
    return target.split("#", 1)[0]


def main() -> int:
    missing: list[tuple[Path, str]] = []

    for path in iter_markdown_files():
        text = path.read_text(encoding="utf-8")
        for match in LINK_RE.finditer(text):
            target = clean_target(match.group(1))
            if not target or is_external(target):
                continue
            full = (path.parent / target).resolve()
            try:
                full.relative_to(ROOT)
            except ValueError:
                missing.append((path, target))
                continue
            if not full.exists():
                missing.append((path, target))

    if missing:
        for path, target in missing:
            print(f"{path.relative_to(ROOT)}: missing local link target {target}")
        return 1

    print(f"checked {len(iter_markdown_files())} markdown files; local links ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())

