#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
TEMPLATE = ROOT / ".github/release-template.md"
PLAN = ROOT / "docs/wiki/roadmaps/local-dmg-distribution-plan.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def main():
    failures = []
    template = TEMPLATE.read_text(encoding="utf-8") if TEMPLATE.is_file() else ""
    plan = PLAN.read_text(encoding="utf-8") if PLAN.is_file() else ""
    template_lower = template.lower()
    plan_lower = plan.lower()

    require(TEMPLATE.is_file(), "release notes template must exist", failures)
    require("## install" in template_lower, "release template must include install steps", failures)
    require("## known issues" in template_lower, "release template must include known issues", failures)
    require("## privacy" in template_lower, "release template must include privacy statement", failures)
    require("shasum -a 256" in template, "release template must include checksum verification command", failures)
    require("original photo files must not be modified" in template_lower, "release template must state original-file safety", failures)
    require("not publish an unsigned developer-preview artifact as user-ready" in template_lower, "release template must preserve unsigned boundary", failures)
    require("auto-update" in template_lower and "homebrew" in template_lower, "release template must state deferred distribution exclusions", failures)
    require(".github/release-template.md" in plan, "local DMG roadmap must reference release template path", failures)
    require("task 9.2" in plan_lower and "completed" in plan_lower, "local DMG roadmap must record Task 9.2 completion", failures)

    if failures:
        for failure in failures:
            print(f"release template check failed: {failure}", file=sys.stderr)
        return 1

    print("release template checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
