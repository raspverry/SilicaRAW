#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RUNBOOK = ROOT / "docs/wiki/roadmaps/local-dmg-release-runbook.md"
README = ROOT / "README.md"
PLAN = ROOT / "docs/wiki/roadmaps/local-dmg-distribution-plan.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def main():
    failures = []
    runbook = RUNBOOK.read_text(encoding="utf-8") if RUNBOOK.is_file() else ""
    readme = README.read_text(encoding="utf-8") if README.is_file() else ""
    plan = PLAN.read_text(encoding="utf-8") if PLAN.is_file() else ""
    runbook_lower = runbook.lower()
    readme_lower = readme.lower()
    plan_lower = plan.lower()

    require(RUNBOOK.is_file(), "local DMG release runbook must exist", failures)
    require("scripts/harness/check.sh" in runbook, "release runbook must include prerelease harness check", failures)
    require("v0.1.0-alpha." in runbook, "release runbook must document alpha tag naming", failures)
    require("developer-preview-" in runbook, "release runbook must document developer preview tag naming", failures)
    require("rollback" in runbook_lower, "release runbook must include rollback steps", failures)
    require("resolving-common-notarization-issues" in runbook, "release runbook must link notarization troubleshooting", failures)
    require("spctl --assess" in runbook, "release runbook must include Gatekeeper assessment commands", failures)
    require("not attach unsigned developer-preview artifacts" in runbook_lower, "release runbook must preserve unsigned release boundary", failures)
    require("local-dmg-release-runbook.md" in readme, "README must link the release runbook", failures)
    require("developer-preview" in readme_lower, "README must mention developer preview distribution status", failures)
    require("task 9.1" in plan_lower and "release runbook" in plan_lower, "local DMG roadmap must record Task 9.1 runbook", failures)

    if failures:
        for failure in failures:
            print(f"release runbook check failed: {failure}", file=sys.stderr)
        return 1

    print("release runbook checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
