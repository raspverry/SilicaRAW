#!/usr/bin/env python3
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REPORT_JSON = ROOT / "docs/wiki/reports/library-scale-benchmarks.json"
REPORT_MD = ROOT / "docs/wiki/reports/library-scale-benchmarks.md"
RUNNER_RS = ROOT / "crates/silica-storage/examples/library_scale_benchmark.rs"
RUNNER_PY = ROOT / "scripts/harness/run-library-scale-benchmarks.py"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def main():
    failures = []
    require(RUNNER_RS.is_file(), "missing Rust library scale benchmark runner", failures)
    require(RUNNER_PY.is_file(), "missing Python library scale benchmark wrapper", failures)
    require(REPORT_JSON.is_file(), "missing library scale benchmark JSON report", failures)
    require(REPORT_MD.is_file(), "missing library scale benchmark markdown report", failures)

    if RUNNER_RS.is_file():
        result = subprocess.run(
            ["cargo", "check", "-p", "silica-storage", "--example", "library_scale_benchmark", "--quiet"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        require(
            result.returncode == 0,
            f"Rust benchmark runner must compile\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}",
            failures,
        )

    report = None
    if REPORT_JSON.is_file():
        try:
            report = json.loads(REPORT_JSON.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            failures.append(f"library scale benchmark JSON is invalid: {error}")

    if isinstance(report, dict):
        machine = report.get("machine", {})
        require(machine.get("os"), "benchmark report must include machine.os", failures)
        require(machine.get("arch"), "benchmark report must include machine.arch", failures)
        require(
            machine.get("cpu_count") is not None,
            "benchmark report must include machine.cpu_count",
            failures,
        )
        require(
            report.get("scope_note")
            and "not universal performance guarantees" in report["scope_note"],
            "benchmark report must state results are not universal guarantees",
            failures,
        )
        datasets = report.get("datasets", [])
        sizes = {dataset.get("photo_count") for dataset in datasets if isinstance(dataset, dict)}
        require(sizes == {1000, 10000, 50000}, "benchmark report must cover 1k, 10k, and 50k datasets", failures)
        for dataset in datasets:
            if not isinstance(dataset, dict):
                failures.append("benchmark dataset entry must be an object")
                continue
            shape = dataset.get("shape", {})
            require(shape.get("jpeg_count") is not None, "dataset shape must include jpeg_count", failures)
            require(shape.get("raw_count") is not None, "dataset shape must include raw_count", failures)
            require(shape.get("unsupported_count") is not None, "dataset shape must include unsupported_count", failures)
            timings = dataset.get("timings", {})
            for key in [
                "query_imported_page_ms",
                "query_filtered_jpeg_ms",
                "query_metadata_dimensions_ms",
                "query_search_ms",
                "render_adjacent_page_model_ms",
            ]:
                value = timings.get(key)
                require(
                    isinstance(value, (int, float)) and value >= 0,
                    f"dataset {dataset.get('photo_count')} missing non-negative timing {key}",
                    failures,
                )

    if REPORT_MD.is_file():
        markdown = REPORT_MD.read_text(encoding="utf-8")
        for marker in [
            "# Library Scale Benchmarks",
            "Machine",
            "Dataset Shape",
            "Timings",
            "not universal performance guarantees",
        ]:
            require(marker in markdown, f"markdown report missing {marker}", failures)

    if failures:
        for failure in failures:
            print(f"library-scale benchmark check failed: {failure}", file=sys.stderr)
        return 1

    print("library scale benchmark report ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
