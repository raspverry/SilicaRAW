#!/usr/bin/env python3
"""Check Display P3 export enablement QA coverage."""

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
CHECKLIST = ROOT / "checklists" / "COLOR_EXPORT_MANUAL_QA.md"
EXPORT_README = ROOT / "crates" / "silica-export" / "README.md"
TASK_CARD = ROOT / "docs" / "wiki" / "tasks" / "20.5-display-p3-export-enablement.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def require_text(source, needle, label, failures):
    require(needle in source, f"{label} must mention {needle!r}", failures)


def main():
    failures = []
    require(CHECKLIST.is_file(), "color export QA checklist is missing", failures)
    require(EXPORT_README.is_file(), "silica-export README is missing", failures)
    require(TASK_CARD.is_file(), "Task 20.5 card is missing", failures)

    checklist = CHECKLIST.read_text(encoding="utf-8") if CHECKLIST.is_file() else ""
    readme = EXPORT_README.read_text(encoding="utf-8") if EXPORT_README.is_file() else ""
    task = TASK_CARD.read_text(encoding="utf-8") if TASK_CARD.is_file() else ""

    for required in [
        "Task 20.5 Display P3 Export Enablement",
        "Display P3 export remains explicit",
        "sRGB remains the default",
        "profile/ICC capability claim",
        "not a visual color-correctness claim",
    ]:
        require_text(checklist, required, "color export QA checklist", failures)

    require_text(
        readme,
        "Display P3 is exposed only as an explicit JPEG export option",
        "silica-export README",
        failures,
    )
    require_text(task, "status: completed", "Task 20.5 card", failures)
    require_text(task, "scripts/harness/check-color-export-qa.py", "Task 20.5 card", failures)

    if failures:
        for failure in failures:
            print(f"color export QA check failed: {failure}", file=sys.stderr)
        return 1

    print("color export QA checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
