#!/usr/bin/env python3
"""Guard the Task 10.5 backup/restore recovery policy."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT / "docs/wiki/topics/backup-restore.md"
ROADMAP = ROOT / "docs/wiki/roadmaps/post-alpha-product-roadmap.md"
DATA_SAFETY = ROOT / "docs/wiki/topics/data-safety.md"


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle}")


def main() -> None:
    policy = POLICY.read_text(encoding="utf-8")
    roadmap = ROADMAP.read_text(encoding="utf-8")
    data_safety = DATA_SAFETY.read_text(encoding="utf-8")

    for needle, label in [
        ("checkpoint-before-copy backup policy", "checkpoint policy"),
        ("Original referenced photo files.", "original exclusion"),
        ("`thumbnails/`.", "thumbnail exclusion"),
        ("`previews/`.", "preview exclusion"),
        ("`render-cache/`.", "render cache exclusion"),
        ("`ai-cache/`.", "AI cache exclusion"),
        ("An empty destination directory.", "empty restore destination"),
        ("rollback copy", "rollback restore path"),
        ("Restore must not:", "restore prohibitions"),
        ("Write into original referenced photo folders.", "original restore prohibition"),
        ("A backup from a newer catalog schema must be rejected", "newer schema rejection"),
        ("If migration fails", "migration failure behavior"),
        ("Task 10.5 implementation tests must prove", "future test policy"),
    ]:
        require(policy, needle, label)

    require(roadmap, "Task 10.5", "roadmap task")
    require(roadmap, "backup/WAL/checkpoint/restore policy", "roadmap policy status")
    require(data_safety, "Backup and Restore", "data safety topic link")
    require(data_safety, "checkpoint-before-copy", "data safety policy summary")

    print("recovery policy ok")


if __name__ == "__main__":
    main()
