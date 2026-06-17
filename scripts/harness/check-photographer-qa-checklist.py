#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CHECKLIST = ROOT / "checklists/PHOTOGRAPHER_WORKFLOW_QA.md"
TASK_CARD = ROOT / "docs/wiki/tasks/22.5-manual-photographer-qa-checklist.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def has_marker(text, marker):
    return marker.lower() in text.lower()


def main():
    failures = []
    require(CHECKLIST.is_file(), "missing photographer workflow QA checklist", failures)
    require(TASK_CARD.is_file(), "missing Task 22.5 task card", failures)

    checklist = CHECKLIST.read_text(encoding="utf-8") if CHECKLIST.is_file() else ""
    task_card = TASK_CARD.read_text(encoding="utf-8") if TASK_CARD.is_file() else ""

    if checklist:
        required_markers = [
            "# Photographer Workflow QA",
            "Licensed or User-Provided Assets",
            "Culling",
            "Metadata",
            "Undo",
            "Develop",
            "Masks",
            "Export",
            "Responsiveness",
            "Data Safety",
            "Color and Export",
            "Known Limitations",
            "Original files unchanged",
            "No private photos are committed",
            "JPEG sRGB",
        ]
        for marker in required_markers:
            require(has_marker(checklist, marker), f"checklist missing {marker}", failures)

        required_task_refs = [
            "Task 22.5",
            "Phases 17 through 21",
            "SILICARAW_RAW_FIXTURE_MANIFEST",
        ]
        for marker in required_task_refs:
            require(marker in checklist, f"checklist missing reference {marker}", failures)

        open_items = checklist.count("- [ ]")
        completed_items = checklist.count("- [x]")
        require(open_items >= 8, "checklist must contain manual open QA steps", failures)
        require(completed_items >= 4, "checklist must record completed static evidence checks", failures)

    if task_card:
        for marker in [
            "status: completed",
            "PHOTOGRAPHER_WORKFLOW_QA.md",
            "licensed or user-provided local assets",
            "Known limitations",
            "scripts/harness/check-photographer-qa-checklist.py",
        ]:
            require(marker in task_card, f"Task 22.5 card missing {marker}", failures)

    if failures:
        for failure in failures:
            print(f"photographer QA checklist check failed: {failure}", file=sys.stderr)
        return 1

    print("photographer QA checklist ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
