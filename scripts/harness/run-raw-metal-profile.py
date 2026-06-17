#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import platform
import re
import resource
import shutil
import statistics
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REPORT_DIR = ROOT / "docs/wiki/reports"
REPORT_JSON = REPORT_DIR / "raw-metal-performance-profile.json"
REPORT_MD = REPORT_DIR / "raw-metal-performance-profile.md"
RUN_COUNT = 3
WARMUP_COUNT = 1


OPERATIONS = [
    {
        "category": "decode_time",
        "label": "RAW decode contract boundary",
        "operation": "cargo test -p silica-decode product_raw_decode_plan_supports_successful_fixture_probe",
        "command": [
            "cargo",
            "test",
            "-p",
            "silica-decode",
            "product_raw_decode_plan_supports_successful_fixture_probe",
            "--quiet",
        ],
        "raw_fixture_class": "synthetic supported fixture probe contract",
        "status": "measured_contract",
        "notes": "Measures the fixture-backed decode planning contract. It does not run Core Image RAW decode bytes unless a legal fixture manifest is provided to the separate ignored tests.",
    },
    {
        "category": "render_time",
        "label": "Viewer render scheduler boundary",
        "operation": "cargo test -p silica-render viewer_render_scheduler_records_latest_request_wins",
        "command": [
            "cargo",
            "test",
            "-p",
            "silica-render",
            "viewer_render_scheduler_records_latest_request_wins",
            "--quiet",
        ],
        "raw_fixture_class": "decoded artifact identity contract",
        "status": "measured_contract",
        "notes": "Measures the render request scheduler boundary only. It does not allocate Metal textures or render pixels.",
    },
    {
        "category": "ui_latency",
        "label": "Feature-gated native viewer request smoke",
        "operation": "cargo test -p silica-desktop --features native-metal-viewer render_request_smoke_evidence_is_reviewable",
        "command": [
            "cargo",
            "test",
            "-p",
            "silica-desktop",
            "--features",
            "native-metal-viewer",
            "render_request_smoke_evidence_is_reviewable",
            "--quiet",
        ],
        "raw_fixture_class": "feature-gated decoded artifact identity",
        "status": "measured_feature_gate",
        "notes": "Measures the feature-gated native viewer request smoke path as a UI-latency proxy. The default app path still keeps native Metal viewer behavior disabled.",
    },
    {
        "category": "export_time",
        "label": "RAW-derived export safety boundary",
        "operation": "cargo test -p silica-core raw_derived_jpeg_srgb_export_rejects_original_overwrite_before_decode",
        "command": [
            "cargo",
            "test",
            "-p",
            "silica-core",
            "raw_derived_jpeg_srgb_export_rejects_original_overwrite_before_decode",
            "--quiet",
        ],
        "raw_fixture_class": "synthetic RAW catalog guard",
        "status": "measured_preflight",
        "notes": "Measures the RAW-derived export safety preflight. Full fixture-backed RAW export timing remains gated on SILICARAW_RAW_FIXTURE_MANIFEST.",
    },
]


def run(command):
    started = time.perf_counter()
    measured_command = command
    parse_time_output = False
    if platform.system() == "Darwin" and shutil.which("/usr/bin/time"):
        measured_command = ["/usr/bin/time", "-l", *command]
        parse_time_output = True
    result = subprocess.run(
        measured_command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    elapsed_ms = (time.perf_counter() - started) * 1000.0
    if result.returncode != 0:
        raise RuntimeError(
            "RAW/Metal profile command failed\n"
            f"command: {' '.join(command)}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    max_rss_kb = None
    if parse_time_output:
        match = re.search(r"(\d+)\s+maximum resident set size", result.stderr)
        if match:
            max_rss_kb = round(int(match.group(1)) / 1024.0, 3)
    if max_rss_kb is None:
        max_rss_kb = child_max_rss_kb()
    return round(elapsed_ms, 3), max_rss_kb


def child_max_rss_kb():
    max_rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if platform.system() == "Darwin":
        return round(max_rss / 1024.0, 3)
    return round(float(max_rss), 3)


def percentile(values, ratio):
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, math.ceil(len(ordered) * ratio) - 1))
    return ordered[index]


def median_ms(values):
    return round(statistics.median(values), 3)


def p95_ms(values):
    return round(percentile(values, 0.95), 3)


def command_output(command):
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def app_version():
    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    in_workspace_package = False
    for line in cargo_toml.splitlines():
        stripped = line.strip()
        if stripped == "[workspace.package]":
            in_workspace_package = True
            continue
        if stripped.startswith("[") and stripped != "[workspace.package]":
            in_workspace_package = False
        if in_workspace_package and stripped.startswith("version"):
            return stripped.split("=", 1)[1].strip().strip('"')
    return "unknown"


def machine_info():
    machine_model = command_output(["sysctl", "-n", "hw.model"]) or platform.machine()
    chip = command_output(["sysctl", "-n", "machdep.cpu.brand_string"]) or platform.processor()
    memory = command_output(["sysctl", "-n", "hw.memsize"])
    if memory is None and hasattr(os, "sysconf"):
        memory = str(os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES"))
    return {
        "os": platform.platform(),
        "macos_version": platform.mac_ver()[0] or None,
        "arch": platform.machine(),
        "machine_model": machine_model,
        "chip": chip or "unknown",
        "memory_bytes": int(memory) if memory and memory.isdigit() else 0,
        "python": platform.python_version(),
        "rustc": command_output(["rustc", "--version"]) or "unknown",
    }


def profile_operation(spec):
    for _ in range(WARMUP_COUNT):
        run(spec["command"])
    measured = [run(spec["command"]) for _ in range(RUN_COUNT)]
    runs = [elapsed_ms for elapsed_ms, _ in measured]
    max_rss_values = [max_rss_kb for _, max_rss_kb in measured]
    return {
        "category": spec["category"],
        "label": spec["label"],
        "operation": spec["operation"],
        "status": spec["status"],
        "raw_fixture_class": spec["raw_fixture_class"],
        "median_ms": median_ms(runs),
        "p95_ms": p95_ms(runs),
        "runs_ms": runs,
        "max_rss_kb": max(max_rss_values),
        "max_rss_runs_kb": max_rss_values,
        "notes": spec["notes"],
    }


def build_report():
    operations = [profile_operation(spec) for spec in OPERATIONS]
    return {
        "schema": "silica.raw_metal_profile.v1",
        "generated_at": dt.datetime.now(dt.timezone.utc).isoformat(),
        "updated_date": dt.date.today().isoformat(),
        "git_commit": command_output(["git", "rev-parse", "--short", "HEAD"]) or "unknown",
        "app_version": app_version(),
        "machine": machine_info(),
        "scope_note": "Local RAW/Metal profiling evidence for this machine only; results are not universal performance guarantees. This report does not implement RAW decoding or Metal viewer behavior.",
        "run_count": RUN_COUNT,
        "warmup_count": WARMUP_COUNT,
        "fixture_manifest": os.environ.get("SILICARAW_RAW_FIXTURE_MANIFEST"),
        "operations": operations,
        "unsupported_or_gated_paths": [
            "Full fixture-backed Core Image RAW decode timing requires SILICARAW_RAW_FIXTURE_MANIFEST and legal local RAW files.",
            "Full fixture-backed RAW-derived JPEG export timing requires SILICARAW_RAW_FIXTURE_MANIFEST and remains outside default harness runs.",
            "Native Metal viewer pixel rendering remains feature-gated; this profile records request-boundary smoke timing, not GPU pixel throughput.",
            "UI latency is represented by the native viewer request smoke boundary; full interactive drag latency still requires an installed-app profiling pass.",
        ],
    }


def markdown_report(report):
    machine = report["machine"]
    lines = [
        "---",
        "title: RAW and Metal Performance Profile",
        "status: active",
        "audience: agents",
        f"updated: {report['updated_date']}",
        "source_of_truth: scripts/harness/run-raw-metal-profile.py",
        "---",
        "",
        "# RAW and Metal Performance Profile",
        "",
        "## Scope",
        "",
        report["scope_note"],
        "",
        "The measured rows use existing contract and feature-gated smoke paths. They keep unsupported fixture-backed RAW decode/export and full Metal pixel rendering visible instead of silently treating proxies as product throughput.",
        "",
        "## Machine",
        "",
        "| Field | Value |",
        "|---|---|",
        f"| Git Commit | `{report['git_commit']}` |",
        f"| App Version | `{report['app_version']}` |",
        f"| OS | `{machine['os']}` |",
        f"| macOS | `{machine['macos_version']}` |",
        f"| Arch | `{machine['arch']}` |",
        f"| Machine Model | `{machine['machine_model']}` |",
        f"| Chip | `{machine['chip']}` |",
        f"| Memory Bytes | `{machine['memory_bytes']}` |",
        f"| Rust | `{machine['rustc']}` |",
        "",
        "## Timings",
        "",
        f"Each row records median and p95 milliseconds over {report['run_count']} local runs after {report['warmup_count']} warm-up run.",
        "",
        "| Category | Operation | Status | RAW Fixture Class | Median ms | p95 ms | Notes |",
        "|---|---|---|---|---:|---:|---|",
    ]
    for operation in report["operations"]:
        lines.append(
            f"| {operation['category']} | {operation['label']} | `{operation['status']}` | "
            f"{operation['raw_fixture_class']} | {operation['median_ms']} | "
            f"{operation['p95_ms']} | {operation['notes']} |"
        )

    lines.extend(
        [
            "",
            "## Memory",
            "",
            "Memory pressure is recorded as child-process `max_rss_kb` observed while running each measured command. On macOS the source value is normalized from bytes to KiB.",
            "",
            "| Category | Operation | max_rss_kb | Runs ms |",
            "|---|---|---:|---|",
        ]
    )
    for operation in report["operations"]:
        lines.append(
            f"| {operation['category']} | {operation['label']} | "
            f"{operation['max_rss_kb']} | `{operation['runs_ms']}` |"
        )

    lines.extend(
        [
            "",
            "## Unsupported and Gated Paths",
            "",
        ]
    )
    for path in report["unsupported_or_gated_paths"]:
        lines.append(f"- {path}")

    lines.extend(
        [
            "",
            "## Reproduce",
            "",
            "```bash",
            "python3 scripts/harness/run-raw-metal-profile.py",
            "python3 scripts/harness/check-raw-metal-profile.py",
            "```",
            "",
        ]
    )
    return "\n".join(lines)


def smoke():
    for spec in OPERATIONS:
        elapsed_ms, max_rss_kb = run(spec["command"])
        print(f"{spec['category']} smoke ok: {elapsed_ms} ms, {max_rss_kb} max_rss_kb")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Run SilicaRAW RAW/Metal profile evidence.")
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="execute each measured command once without writing report artifacts",
    )
    args = parser.parse_args()
    if args.smoke:
        return smoke()

    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    report = build_report()
    REPORT_JSON.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    REPORT_MD.write_text(markdown_report(report), encoding="utf-8")
    print(f"RAW/Metal profile JSON: {REPORT_JSON.relative_to(ROOT)}")
    print(f"RAW/Metal profile report: {REPORT_MD.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
