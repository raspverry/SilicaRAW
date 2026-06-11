#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
POLICY = ROOT / "checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md"
RAW_DOC = ROOT / "docs/wiki/topics/raw-decoding.md"
COLOR_DOC = ROOT / "docs/wiki/topics/color-management.md"
ROADMAP = ROOT / "docs/wiki/roadmaps/post-alpha-product-roadmap.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def read_text(path, failures):
    try:
        return path.read_text(encoding="utf-8")
    except Exception as exc:
        failures.append(f"failed to read {path.relative_to(ROOT)}: {exc}")
        return ""


def task_section(text, heading, next_heading):
    start = text.find(heading)
    if start == -1:
        return ""
    end = text.find(next_heading, start + len(heading))
    return text[start:] if end == -1 else text[start:end]


def main():
    failures = []
    policy = read_text(POLICY, failures)
    raw_doc = read_text(RAW_DOC, failures)
    color_doc = read_text(COLOR_DOC, failures)
    roadmap = read_text(ROADMAP, failures)

    for required in [
        "Byte Equality",
        "File and Profile Inspection",
        "Pixel or Perceptual Tolerance",
        "Manual Visual Review",
        "Forbidden now",
        "Future Graduation Gates",
    ]:
        require(required in policy, f"policy must include {required}", failures)

    for forbidden_claim in [
        "SilicaRAW supports RAW decoding.",
        "SilicaRAW is color correct.",
        "SilicaRAW has a validated golden image baseline.",
    ]:
        require(forbidden_claim in policy, f"policy must forbid claim: {forbidden_claim}", failures)

    require("no automated pixel comparison may be used as evidence for color correctness" in policy, "policy must block pixel proof without approved tolerances", failures)
    require("viewer used: Preview.app or Photos" in policy, "policy must require Preview.app or Photos manual review", failures)
    require("viewer used: Preview.app, Photos, or SilicaRAW" not in policy, "policy must not allow SilicaRAW as the manual review viewer", failures)
    require("RAW support claims require" in policy, "policy must define RAW support graduation gates", failures)
    require("Color correctness claims require" in policy, "policy must define color correctness graduation gates", failures)
    require("Task 10.2 does not add RAW decoding" in raw_doc, "RAW docs must preserve Task 10.2 no-decoding boundary", failures)
    require("Color correctness claims remain blocked" in color_doc, "color docs must preserve color claim boundary", failures)
    require("Fixture-backed golden image baseline" in color_doc, "color docs must keep golden baseline blocked until evidence exists", failures)
    require("GOLDEN_IMAGE_TOLERANCE_POLICY.md" in raw_doc, "RAW docs must link tolerance policy", failures)
    require("GOLDEN_IMAGE_TOLERANCE_POLICY.md" in color_doc, "color docs must link tolerance policy", failures)
    task_10_2 = task_section(
        roadmap,
        "### Task 10.2: Golden Image and Tolerance Policy",
        "### Task 10.3: Sidecar v1 Read/Write Foundation",
    )
    require(task_10_2, "roadmap must list Task 10.2", failures)
    require("**Status:** Completed on 2026-06-11" in task_10_2, "roadmap must mark Task 10.2 completed", failures)

    if failures:
        for failure in failures:
            print(f"golden tolerance policy check failed: {failure}", file=sys.stderr)
        return 1

    print("golden image tolerance policy ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
