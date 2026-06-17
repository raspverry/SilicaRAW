#!/usr/bin/env python3
import json
import os
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REPORT_DIR = ROOT / "docs/wiki/reports"
REPORT_JSON = REPORT_DIR / "library-scale-benchmarks.json"
REPORT_MD = REPORT_DIR / "library-scale-benchmarks.md"
WORKDIR = ROOT / ".tmp/library-scale-benchmark"


def run_benchmark():
    env = os.environ.copy()
    env["SILICARAW_BENCHMARK_WORKDIR"] = str(WORKDIR.relative_to(ROOT))
    result = subprocess.run(
        ["cargo", "run", "-p", "silica-storage", "--example", "library_scale_benchmark", "--quiet"],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            "library scale benchmark failed\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    return json.loads(result.stdout)


def markdown_report(report):
    machine = report["machine"]
    lines = [
        "---",
        "title: Library Scale Benchmarks",
        "status: active",
        "audience: agents",
        "updated: 2026-06-17",
        "source_of_truth: scripts/harness/run-library-scale-benchmarks.py",
        "---",
        "",
        "# Library Scale Benchmarks",
        "",
        "## Scope",
        "",
        report["scope_note"],
        "",
        "The benchmark seeds synthetic catalog rows in a temporary local library, then measures the existing typed `silica-storage::query_library_photos` path. It does not create original photo files, decode RAW files, generate thumbnails, or measure the native viewer.",
        "",
        "## Machine",
        "",
        "| Field | Value |",
        "|---|---|",
        f"| OS | `{machine['os']}` |",
        f"| Arch | `{machine['arch']}` |",
        f"| CPU Count | `{machine['cpu_count']}` |",
        f"| Rust | `{machine['rustc']}` |",
        "",
        "## Dataset Shape",
        "",
        "| Photos | JPEG | RAW | Unsupported | With Dimensions | Picked | Rejected |",
        "|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for dataset in report["datasets"]:
        shape = dataset["shape"]
        lines.append(
            f"| {dataset['photo_count']} | {shape['jpeg_count']} | {shape['raw_count']} | "
            f"{shape['unsupported_count']} | {shape['metadata_dimensions_count']} | "
            f"{shape['picked_count']} | {shape['rejected_count']} |"
        )

    lines.extend(
        [
            "",
            "## Timings",
            "",
            "All values are median milliseconds over the recorded query runs. The render-adjacent row is a lightweight page-model shaping pass over the queried page, not GPU rendering.",
            "",
            "| Photos | Imported Page | JPEG Filter | Metadata Filter | Search | Render-Adjacent Page Model | Seed Catalog |",
            "|---:|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for dataset in report["datasets"]:
        timings = dataset["timings"]
        lines.append(
            f"| {dataset['photo_count']} | {timings['query_imported_page_ms']} | "
            f"{timings['query_filtered_jpeg_ms']} | {timings['query_metadata_dimensions_ms']} | "
            f"{timings['query_search_ms']} | {timings['render_adjacent_page_model_ms']} | "
            f"{dataset['seed_catalog_ms']} |"
        )

    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "python3 scripts/harness/run-library-scale-benchmarks.py",
            "python3 scripts/harness/check-library-scale-benchmark.py",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def main():
    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    report = run_benchmark()
    REPORT_JSON.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    REPORT_MD.write_text(markdown_report(report), encoding="utf-8")
    print(f"library scale benchmark JSON: {REPORT_JSON.relative_to(ROOT)}")
    print(f"library scale benchmark report: {REPORT_MD.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
