#!/usr/bin/env python3
import json
import math
import statistics
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REPORT_JSON = ROOT / "docs/wiki/reports/raw-metal-performance-profile.json"
REPORT_MD = ROOT / "docs/wiki/reports/raw-metal-performance-profile.md"
RUNNER_PY = ROOT / "scripts/harness/run-raw-metal-profile.py"
CHECKLIST = ROOT / "checklists/RAW_METAL_PERFORMANCE_CHECKLIST.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def percentile(values, ratio):
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(len(ordered) * ratio) - 1))
    return ordered[index]


def rounded_median(values):
    return round(statistics.median(values), 3)


def rounded_p95(values):
    return round(percentile(values, 0.95), 3)


def main():
    failures = []
    require(RUNNER_PY.is_file(), "missing RAW/Metal profile runner", failures)
    require(REPORT_JSON.is_file(), "missing RAW/Metal profile JSON report", failures)
    require(REPORT_MD.is_file(), "missing RAW/Metal profile markdown report", failures)
    require(CHECKLIST.is_file(), "missing RAW/Metal performance checklist", failures)
    if RUNNER_PY.is_file():
        result = subprocess.run(
            ["python3", str(RUNNER_PY.relative_to(ROOT)), "--smoke"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        require(
            result.returncode == 0,
            f"RAW/Metal profile runner smoke failed\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}",
            failures,
        )

    report = None
    if REPORT_JSON.is_file():
        try:
            report = json.loads(REPORT_JSON.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            failures.append(f"RAW/Metal profile JSON is invalid: {error}")

    if isinstance(report, dict):
        require(
            report.get("schema") == "silica.raw_metal_profile.v1",
            "RAW/Metal profile report must use schema silica.raw_metal_profile.v1",
            failures,
        )
        require(report.get("git_commit"), "RAW/Metal profile report must include git_commit", failures)
        require(report.get("app_version"), "RAW/Metal profile report must include app_version", failures)
        require(report.get("run_count") == 3, "RAW/Metal profile report must include three measured runs", failures)
        require(report.get("warmup_count") == 1, "RAW/Metal profile report must include one warm-up run", failures)
        updated_date = report.get("updated_date")
        require(updated_date, "RAW/Metal profile report must include updated_date", failures)
        scope_note = report.get("scope_note", "")
        require(
            "not universal performance guarantees" in scope_note,
            "RAW/Metal profile report must reject universal performance claims",
            failures,
        )
        require(
            "does not implement RAW decoding or Metal viewer behavior" in scope_note,
            "RAW/Metal profile report must keep implementation scope explicit",
            failures,
        )
        machine = report.get("machine", {})
        for key in ["os", "machine_model", "chip", "memory_bytes"]:
            require(machine.get(key) is not None, f"RAW/Metal profile machine missing {key}", failures)

        operations = report.get("operations", [])
        require(isinstance(operations, list), "RAW/Metal profile operations must be a list", failures)
        categories = {
            operation.get("category")
            for operation in operations
            if isinstance(operation, dict)
        }
        require(
            categories == {"decode_time", "render_time", "ui_latency", "export_time"},
            "RAW/Metal profile must separate decode_time, render_time, ui_latency, and export_time",
            failures,
        )
        for category in categories:
            require(
                sum(1 for operation in operations if isinstance(operation, dict) and operation.get("category") == category) == 1,
                f"RAW/Metal profile must include exactly one {category} operation",
                failures,
            )
        for operation in operations:
            if not isinstance(operation, dict):
                failures.append("RAW/Metal profile operation entry must be an object")
                continue
            label = operation.get("label", "<unknown>")
            for key in ["category", "status", "raw_fixture_class", "operation", "notes"]:
                require(operation.get(key), f"{label} missing {key}", failures)
            for key in ["median_ms", "p95_ms", "max_rss_kb"]:
                value = operation.get(key)
                require(
                    isinstance(value, (int, float)) and value >= 0,
                    f"{label} missing non-negative {key}",
                    failures,
                )
            require(
                operation.get("runs_ms"),
                f"{label} must include raw run timings",
                failures,
            )
            runs_ms = operation.get("runs_ms")
            max_rss_runs_kb = operation.get("max_rss_runs_kb")
            if isinstance(runs_ms, list):
                require(
                    len(runs_ms) == report.get("run_count"),
                    f"{label} runs_ms length must match run_count",
                    failures,
                )
                require(
                    all(isinstance(value, (int, float)) and value >= 0 for value in runs_ms),
                    f"{label} runs_ms must contain non-negative numbers",
                    failures,
                )
                if runs_ms:
                    require(
                        operation.get("median_ms") == rounded_median(runs_ms),
                        f"{label} median_ms must match runs_ms",
                        failures,
                    )
                    require(
                        operation.get("p95_ms") == rounded_p95(runs_ms),
                        f"{label} p95_ms must match runs_ms",
                        failures,
                    )
            else:
                failures.append(f"{label} runs_ms must be a list")
            if isinstance(max_rss_runs_kb, list):
                require(
                    len(max_rss_runs_kb) == report.get("run_count"),
                    f"{label} max_rss_runs_kb length must match run_count",
                    failures,
                )
                require(
                    all(isinstance(value, (int, float)) and value >= 0 for value in max_rss_runs_kb),
                    f"{label} max_rss_runs_kb must contain non-negative numbers",
                    failures,
                )
                if max_rss_runs_kb:
                    require(
                        operation.get("max_rss_kb") == max(max_rss_runs_kb),
                        f"{label} max_rss_kb must match max_rss_runs_kb",
                        failures,
                    )
            else:
                failures.append(f"{label} max_rss_runs_kb must be a list")

        unsupported = report.get("unsupported_or_gated_paths", [])
        require(
            isinstance(unsupported, list) and unsupported,
            "RAW/Metal profile must record unsupported or gated paths",
            failures,
        )

    if REPORT_MD.is_file():
        markdown = REPORT_MD.read_text(encoding="utf-8")
        for marker in [
            "# RAW and Metal Performance Profile",
            "Scope",
            "Machine",
            "Timings",
            "Memory",
            "Unsupported and Gated Paths",
            "not universal performance guarantees",
        ]:
            require(marker in markdown, f"RAW/Metal profile markdown missing {marker}", failures)
        if isinstance(report, dict) and report.get("updated_date"):
            require(
                f"updated: {report['updated_date']}" in markdown,
                "RAW/Metal profile markdown updated date must match JSON updated_date",
                failures,
            )

    if CHECKLIST.is_file():
        checklist = CHECKLIST.read_text(encoding="utf-8")
        for marker in [
            "# RAW Metal Performance Checklist",
            "Fixture-backed RAW",
            "Native viewer",
            "Known limitations",
        ]:
            require(marker in checklist, f"RAW/Metal checklist missing {marker}", failures)

    if failures:
        for failure in failures:
            print(f"RAW/Metal profile check failed: {failure}", file=sys.stderr)
        return 1

    print("RAW/Metal profile report ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
