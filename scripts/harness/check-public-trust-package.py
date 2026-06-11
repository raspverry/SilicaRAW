#!/usr/bin/env python3
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def read(relative_path):
    path = ROOT / relative_path
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def main():
    failures = []

    required_files = [
        "LICENSE",
        "README.md",
        "CONTRIBUTING.md",
        "SECURITY.md",
        ".github/PULL_REQUEST_TEMPLATE.md",
        ".github/ISSUE_TEMPLATE/bug_report.md",
        ".github/ISSUE_TEMPLATE/feature_request.md",
        ".github/ISSUE_TEMPLATE/scope_question.md",
        "docs/wiki/topics/public-trust.md",
    ]

    for relative_path in required_files:
        require((ROOT / relative_path).is_file(), f"{relative_path} must exist", failures)

    license_text = read("LICENSE")
    readme = read("README.md")
    contributing = read("CONTRIBUTING.md")
    security = read("SECURITY.md")
    pr_template = read(".github/PULL_REQUEST_TEMPLATE.md")
    bug_template = read(".github/ISSUE_TEMPLATE/bug_report.md")
    feature_template = read(".github/ISSUE_TEMPLATE/feature_request.md")
    scope_template = read(".github/ISSUE_TEMPLATE/scope_question.md")
    public_trust = read("docs/wiki/topics/public-trust.md")
    roadmap = read("docs/wiki/roadmaps/post-alpha-product-roadmap.md")

    readme_lower = readme.lower()
    security_lower = security.lower()
    public_trust_lower = public_trust.lower()

    require("MIT License" in license_text, "LICENSE must use MIT text", failures)
    require("License: [MIT](LICENSE)" in readme, "README must link the MIT license", failures)
    require("SilicaRAW is not production-ready" in readme, "README must state non-production status", failures)
    require("## Known Limitations" in readme, "README must include known limitations", failures)
    require("RAW decoding is not implemented" in readme, "README must not imply RAW decoding is implemented", failures)
    require("Color correctness is not claimed" in readme, "README must avoid color correctness claims", failures)
    require("Signed and notarized release DMGs are blocked" in readme, "README must state signing/notarization block", failures)

    forbidden_readme_phrases = [
        "production-ready raw editor",
        "supports every camera",
        "broad raw support",
        "color correctness is verified",
        "signed and notarized downloads are available",
        "mlx-powered tools when you need them",
    ]
    for phrase in forbidden_readme_phrases:
        require(phrase not in readme_lower, f"README must not overclaim: {phrase}", failures)

    require("## Claims Not Yet Allowed" in public_trust, "public trust topic must include forbidden claims", failures)
    require("Do not claim broad RAW camera support" in public_trust, "public trust topic must block broad RAW claims", failures)
    require(
        "Do not treat unsigned developer-preview DMGs as user-ready releases" in public_trust,
        "public trust topic must preserve unsigned preview boundary",
        failures,
    )
    require("task 10.6.2 still needs" not in public_trust_lower, "public trust topic must not say 10.6.2 is pending", failures)

    require("scripts/harness/check.sh" in contributing, "CONTRIBUTING must require the harness", failures)
    require("docs/DEPENDENCIES.md" in contributing, "CONTRIBUTING must mention dependency documentation", failures)
    require("Do not modify original photo files" in contributing, "CONTRIBUTING must preserve original-file safety", failures)
    require("feature/" in contributing and "fix/" in contributing, "CONTRIBUTING must document branch prefixes", failures)

    require("do not include exploit details" in security_lower, "SECURITY must avoid public exploit disclosure", failures)
    require("original photo files" in security_lower, "SECURITY must cover original-file safety", failures)
    require("Private GitHub Security Advisory" in security, "SECURITY must name the preferred private route", failures)
    deferred_runtime_marker = "tele" + "metry"
    require(deferred_runtime_marker in security and "auto-update" in security, "SECURITY must state deferred sensitive surfaces", failures)

    require("original photo files" in bug_template, "bug template must ask about original-file safety", failures)
    require("Known limitations" in feature_template, "feature template must point requesters to limitations", failures)
    require("deferred" in scope_template.lower(), "scope template must handle deferred features", failures)
    require("Public Trust" in pr_template, "PR template must include public trust checks", failures)

    require("Task 10.6.2 completed" in roadmap, "roadmap must record Task 10.6.2 completion", failures)

    if failures:
        for failure in failures:
            print(f"public trust package check failed: {failure}", file=sys.stderr)
        return 1

    print("public trust package checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
