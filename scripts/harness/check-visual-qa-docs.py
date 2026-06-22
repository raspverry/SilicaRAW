#!/usr/bin/env python3
from __future__ import annotations

import ast
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RUNNER = ROOT / "scripts/harness/run-final-visual-qa.py"
TOPIC = ROOT / "docs/wiki/topics/ui-visual-responsive-qa.md"
PLAN = ROOT / "docs/wiki/roadmaps/local-alpha-quality-closure-plan.md"


def literal_assignment(name: str):
    tree = ast.parse(RUNNER.read_text(encoding="utf-8"))
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        if any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
            return ast.literal_eval(node.value)
    raise RuntimeError(f"{RUNNER.relative_to(ROOT)} missing {name}")


def surface_id(surface_name: str) -> str:
    match = re.match(r"^(M\d{3})-", surface_name)
    if not match:
        raise RuntimeError(f"surface name must start with M###-: {surface_name}")
    return match.group(1)


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def main() -> int:
    surfaces = literal_assignment("SURFACES")
    viewports = literal_assignment("VIEWPORTS")
    topic = TOPIC.read_text(encoding="utf-8")
    plan = PLAN.read_text(encoding="utf-8")

    surface_names = [entry[0] for entry in surfaces]
    surface_ids = [surface_id(name) for name in surface_names]
    viewport_dims = [f"{width}x{height}" for _, width, height in viewports]
    surface_count = len(surface_names)
    screenshot_count = surface_count * len(viewports)
    latest_surface_id = max(surface_ids, key=lambda value: int(value[1:]))

    failures: list[str] = []
    for path, text in [(TOPIC, topic), (PLAN, plan)]:
        relative = path.relative_to(ROOT)
        require(
            f"{surface_count} surfaces" in text,
            f"{relative} must record current final visual QA surface count: {surface_count}",
            failures,
        )
        require(
            f"{screenshot_count} screenshots" in text,
            f"{relative} must record current final visual QA screenshot count: {screenshot_count}",
            failures,
        )
        for dims in viewport_dims:
            require(dims in text, f"{relative} must record final visual QA viewport {dims}", failures)
        require(
            latest_surface_id in text,
            f"{relative} must mention latest final visual QA surface id {latest_surface_id}",
            failures,
        )

    for surface in surface_ids:
        require(surface in topic, f"{TOPIC.relative_to(ROOT)} must mention final visual QA surface {surface}", failures)

    if failures:
        for failure in failures:
            print(f"visual QA docs drift: {failure}", file=sys.stderr)
        return 1

    print(
        f"visual QA docs ok: {surface_count} surfaces, {len(viewports)} viewports, {screenshot_count} screenshots"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
