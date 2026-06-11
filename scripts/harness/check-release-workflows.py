#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/developer-preview-macos.yml"
ADR = ROOT / "docs/wiki/decisions/adr-0006-unsigned-developer-preview-dmg.md"
PLAN = ROOT / "docs/wiki/roadmaps/local-dmg-distribution-plan.md"
RUNBOOK = ROOT / "docs/wiki/roadmaps/developer-preview-artifact-runbook.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def main():
    failures = []
    workflow = WORKFLOW.read_text(encoding="utf-8") if WORKFLOW.is_file() else ""
    adr = ADR.read_text(encoding="utf-8") if ADR.is_file() else ""
    plan = PLAN.read_text(encoding="utf-8") if PLAN.is_file() else ""
    runbook = RUNBOOK.read_text(encoding="utf-8") if RUNBOOK.is_file() else ""

    require(WORKFLOW.is_file(), "developer preview workflow must exist", failures)
    require(ADR.is_file(), "ADR 0006 must exist", failures)
    require(RUNBOOK.is_file(), "developer preview artifact runbook must exist", failures)
    require("workflow_dispatch:" in workflow, "developer preview workflow must support manual dispatch", failures)
    require('developer-preview-*' in workflow, "developer preview workflow must use developer-preview tags", failures)
    require("--no-sign" in workflow, "developer preview workflow must build unsigned artifacts explicitly", failures)
    require("actions/upload-artifact@v4" in workflow, "developer preview workflow must upload artifacts with v4", failures)
    require("UNSIGNED-DEVELOPER-PREVIEW.txt" in workflow, "developer preview artifact must include unsigned warning", failures)
    require("APPLE_CERTIFICATE" not in workflow, "developer preview workflow must not require Apple signing secrets", failures)
    require("APPLE_PASSWORD" not in workflow, "developer preview workflow must not require Apple notarization secrets", failures)
    require("not signed or notarized" in workflow, "developer preview artifact warning must mention unsigned state", failures)
    adr_lower = adr.lower()
    plan_lower = plan.lower()
    runbook_lower = runbook.lower()
    require("unsigned" in adr_lower and "notarized" in adr_lower, "ADR 0006 must state unsigned/notarization boundary", failures)
    require("developer id funding" in plan_lower, "local DMG roadmap must record Developer ID funding blocker", failures)
    require("developer-preview" in plan_lower or "developer preview" in plan_lower, "local DMG roadmap must record developer preview path", failures)
    require("gh workflow run" in runbook, "developer preview runbook must document manual workflow dispatch", failures)
    require("developer-preview-" in runbook, "developer preview runbook must document preview tag naming", failures)
    require("shasum -a 256" in runbook, "developer preview runbook must document checksum verification", failures)
    require(
        "not signed" in runbook_lower and "notarized" in runbook_lower,
        "developer preview runbook must preserve unsigned/notarized boundary",
        failures,
    )
    require("not user-ready" in runbook_lower, "developer preview runbook must state artifact is not user-ready", failures)

    if failures:
        for failure in failures:
            print(f"release workflow check failed: {failure}", file=sys.stderr)
        return 1

    print("release workflow checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
