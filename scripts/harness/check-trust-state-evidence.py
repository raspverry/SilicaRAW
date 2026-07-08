#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_SCRATCH = ROOT / ".tmp/q5-trust-states"
DEFAULT_APP = ROOT / "target/release/bundle/macos/SilicaRAW.app"
GENERATOR = ROOT / "scripts/harness/generate-legal-fixtures.py"
EXPECTED_CACHE_DIRS = ["thumbnails", "previews", "render-cache", "ai-cache"]


def relative_path(path):
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sha256_tree(path):
    digest = hashlib.sha256()
    file_count = 0
    byte_count = 0
    for file_path in sorted(item for item in path.rglob("*") if item.is_file()):
        relative = file_path.relative_to(path).as_posix()
        data = file_path.read_bytes()
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(hashlib.sha256(data).hexdigest().encode("ascii"))
        digest.update(b"\0")
        file_count += 1
        byte_count += len(data)
    return digest.hexdigest(), file_count, byte_count


def artifact_record(path):
    if path.is_dir():
        digest, file_count, byte_count = sha256_tree(path)
        kind = "directory"
    else:
        digest = sha256_file(path)
        file_count = 1
        byte_count = path.stat().st_size
        kind = "file"
    return {
        "path": relative_path(path),
        "kind": kind,
        "sha256": digest,
        "file_count": file_count,
        "size_bytes": byte_count,
    }


def host_record():
    return {
        "system": platform.system(),
        "machine": platform.machine(),
        "platform": platform.platform(),
        "macos_version": platform.mac_ver()[0] or "not-macos",
    }


def run(command, env=None):
    return subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def git_commit():
    result = run(["git", "rev-parse", "HEAD"])
    return result.stdout.strip() if result.returncode == 0 else None


def command_tail(output, limit=80):
    return output.splitlines()[-limit:]


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def generate_fixtures(fixtures):
    return run(
        [
            "python3",
            relative_path(GENERATOR),
            "--output",
            relative_path(fixtures),
            "--include-raw-placeholders",
        ]
    )


def parse_args():
    parser = argparse.ArgumentParser(
        description="Record Q5.5 trust-state evidence for supported PNG, unsupported sources, missing originals, and cache clear."
    )
    parser.add_argument(
        "--app",
        type=Path,
        default=DEFAULT_APP,
        help="Path to the built SilicaRAW.app artifact.",
    )
    parser.add_argument(
        "--scratch",
        type=Path,
        default=DEFAULT_SCRATCH,
        help="Scratch directory for fixtures and runtime output.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_SCRATCH / "trust-state-evidence.json",
        help="JSON evidence report output path.",
    )
    return parser.parse_args()


def check_report(report):
    png = report.get("supported_source_sanity", {}).get("png", {})
    unsupported = report.get("unsupported_states", {})
    raw = unsupported.get("raw", {})
    text = unsupported.get("text", {})
    missing = report.get("missing_original_state", {})
    cache = report.get("cache_clear_state", {})
    checks = {
        "supported_png_ready": png.get("file_type") == "PNG"
        and png.get("unsupported") is False
        and png.get("missing") is False
        and png.get("thumbnail_bytes_present") is True
        and png.get("preview_status") == "Ready"
        and png.get("preview_bytes_present") is True,
        "raw_unsupported_blocked": raw.get("unsupported") is True
        and raw.get("missing") is False
        and raw.get("thumbnail_bytes_present") is False
        and raw.get("preview_status") == "Unsupported",
        "text_unsupported_blocked": text.get("unsupported") is True
        and text.get("missing") is False
        and text.get("thumbnail_bytes_present") is False
        and text.get("preview_status") == "Unsupported",
        "missing_original_grid_downgraded": missing.get("thumbnail_ready_before_delete") is True
        and missing.get("source_intentionally_deleted") is True
        and missing.get("grid_missing") is True
        and missing.get("grid_thumbnail_bytes_present") is False,
        "missing_original_preview_histogram_develop_blocked": missing.get("preview_status")
        == "BlockedByDecode"
        and missing.get("histogram_status") == "Missing"
        and missing.get("histogram_pixel_count") == 0
        and missing.get("develop_preview_status") == "BlockedByDecode"
        and missing.get("develop_preview_bytes_present") is False,
        "missing_original_write_paths_blocked": missing.get("commit_error_kind")
        == "unsupportedEdit"
        and missing.get("export_error_kind") == "exportBlocked"
        and missing.get("blocked_export_path_exists") is False,
        "cache_clear_scope_correct": cache.get("cleared_directories") == EXPECTED_CACHE_DIRS
        and cache.get("recreated_directories") == EXPECTED_CACHE_DIRS
        and cache.get("removed_cache_records", 0) > 0
        and cache.get("disposable_cache_removed") is True
        and cache.get("protected_files_preserved") is True
        and cache.get("tracked_originals_unchanged") is True,
    }
    return checks


def main():
    args = parse_args()
    app = args.app if args.app.is_absolute() else ROOT / args.app
    scratch = args.scratch if args.scratch.is_absolute() else ROOT / args.scratch
    output = args.output if args.output.is_absolute() else ROOT / args.output
    fixtures = scratch / "fixtures"

    failures = []
    if not app.is_dir():
        print(f"app artifact does not exist: {app}", file=sys.stderr)
        return 1

    if scratch.exists():
        shutil.rmtree(scratch)
    fixtures.mkdir(parents=True)
    output.parent.mkdir(parents=True, exist_ok=True)

    generator = generate_fixtures(fixtures)
    if generator.returncode != 0:
        print("trust state evidence failed: fixture generation failed", file=sys.stderr)
        print(generator.stdout, file=sys.stderr)
        print(generator.stderr, file=sys.stderr)
        return 1

    env = os.environ.copy()
    env["SILICARAW_TRUST_STATE_FIXTURES"] = str(fixtures)
    env["SILICARAW_TRUST_STATE_OUTPUT"] = str(output)
    command = [
        "cargo",
        "test",
        "-p",
        "silica-desktop",
        "tests::desktop_trust_state_evidence_smoke",
        "--",
        "--exact",
        "--nocapture",
    ]
    smoke = run(command, env=env)
    smoke_output = smoke.stdout + smoke.stderr
    marker_present = "q5 trust state evidence smoke complete" in smoke_output
    if smoke.returncode != 0:
        failures.append("trust state smoke command failed")
    if "desktop_trust_state_evidence_smoke" not in smoke_output:
        failures.append("trust state smoke test name missing from output")
    if not marker_present:
        failures.append("trust state smoke completion marker missing")
    if not output.is_file():
        failures.append("trust state evidence JSON missing")
        report = {}
    else:
        report = load_json(output)

    checks = check_report(report)
    failures.extend(check for check, passed in checks.items() if not passed)
    manifest = load_json(fixtures / "fixture-manifest.json")
    report = {
        **report,
        "generated_at": datetime.now(timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "source_commit": git_commit(),
        "app_artifact": artifact_record(app),
        "host": host_record(),
        "command": command,
        "fixture_manifest": {
            "path": relative_path(fixtures / "fixture-manifest.json"),
            "schema_version": manifest.get("schema_version"),
            "fixture_count": len(manifest.get("fixtures", [])),
            "include_raw_placeholders": manifest.get("include_raw_placeholders", False),
            "source_policy": manifest.get("source_policy"),
        },
        "rust_smoke": {
            "returncode": smoke.returncode,
            "completion_marker_present": marker_present,
            "stdout_stderr_tail": command_tail(smoke_output),
        },
        "checks": checks,
        "failures": failures,
    }
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if failures:
        print(f"trust state evidence failed; report written to {relative_path(output)}", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(f"trust state evidence report written to {relative_path(output)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
