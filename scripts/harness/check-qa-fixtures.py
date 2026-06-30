#!/usr/bin/env python3
import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
GENERATOR = ROOT / "scripts/harness/generate-legal-fixtures.py"
PREFLIGHT = ROOT / "scripts/harness/installed-app-preflight.py"
SCRATCH = ROOT / ".tmp/harness/qa-fixtures"
FIXTURES = SCRATCH / "fixtures"
PREFLIGHT_DIR = SCRATCH / "preflight"
REPORT = PREFLIGHT_DIR / "installed-app-preflight.json"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def run(command, failures):
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    if result.returncode != 0:
        failures.append(
            f"command failed: {' '.join(str(part) for part in command)}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def load_json(path, failures):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        failures.append(f"failed to read JSON {path}: {exc}")
        return {}


def main():
    failures = []
    require(GENERATOR.is_file(), f"missing {GENERATOR.relative_to(ROOT)}", failures)
    require(PREFLIGHT.is_file(), f"missing {PREFLIGHT.relative_to(ROOT)}", failures)

    if failures:
        for failure in failures:
            print(f"qa fixture check failed: {failure}", file=sys.stderr)
        return 1

    if SCRATCH.exists():
        shutil.rmtree(SCRATCH)
    FIXTURES.mkdir(parents=True)
    PREFLIGHT_DIR.mkdir(parents=True)

    run(
        [
            "python3",
            str(GENERATOR.relative_to(ROOT)),
            "--output",
            str(FIXTURES.relative_to(ROOT)),
            "--include-raw-placeholders",
        ],
        failures,
    )

    manifest_path = FIXTURES / "fixture-manifest.json"
    require(manifest_path.is_file(), "fixture generator must write fixture-manifest.json", failures)
    manifest = load_json(manifest_path, failures) if manifest_path.is_file() else {}
    fixtures = manifest.get("fixtures", [])
    hashes = manifest.get("expected_source_hashes", {})

    require(manifest.get("schema_version") == 1, "fixture manifest schema_version must be 1", failures)
    require("synthetic" in manifest.get("license", "").lower(), "fixture manifest must identify synthetic fixture licensing", failures)
    require(isinstance(fixtures, list) and fixtures, "fixture manifest must include fixtures", failures)
    require(isinstance(hashes, dict) and hashes, "fixture manifest must include expected_source_hashes", failures)

    roles = {fixture.get("role") for fixture in fixtures}
    extensions = {Path(fixture.get("relative_path", "")).suffix.lower() for fixture in fixtures}
    require("supported-jpeg" in roles, "fixture set must include supported JPEG/JPG samples", failures)
    require("supported-png" in roles, "fixture set must include supported PNG samples", failures)
    require("supported-tiff" in roles, "fixture set must include supported TIFF samples", failures)
    require({".jpg", ".jpeg"}.issubset(extensions), "fixture set must include both .jpg and .jpeg samples", failures)
    require(".png" in extensions, "fixture set must include a .png sample", failures)
    require(".tiff" in extensions, "fixture set must include a .tiff sample", failures)
    require("unsupported" in roles, "fixture set must include unsupported files", failures)
    require("raw-blocked-placeholder" in roles, "fixture set must include optional RAW-blocked placeholders when requested", failures)

    for fixture in fixtures:
        relative_path = fixture.get("relative_path")
        expected_sha = fixture.get("sha256")
        file_path = FIXTURES / relative_path if relative_path else None
        require(relative_path in hashes, f"{relative_path} missing from expected_source_hashes", failures)
        require(hashes.get(relative_path) == expected_sha, f"{relative_path} hash map mismatch", failures)
        require(file_path is not None and file_path.is_file(), f"{relative_path} file missing", failures)
        if file_path and file_path.is_file():
            data = file_path.read_bytes()
            if fixture.get("role") == "supported-jpeg":
                require(data.startswith(b"\xff\xd8") and data.endswith(b"\xff\xd9"), f"{relative_path} must look like a JPEG", failures)
            if fixture.get("role") == "supported-png":
                require(data.startswith(b"\x89PNG\r\n\x1a\n"), f"{relative_path} must look like a PNG", failures)
            if fixture.get("role") == "supported-tiff":
                require(data.startswith((b"II*\x00", b"MM\x00*")), f"{relative_path} must look like a TIFF", failures)
            if fixture.get("role") == "raw-blocked-placeholder":
                require(b"RAW placeholder" in data, f"{relative_path} must be an explicit placeholder, not a camera RAW file", failures)

    fake_app = PREFLIGHT_DIR / "SilicaRAW.app"
    executable = fake_app / "Contents/MacOS/SilicaRAW"
    executable.parent.mkdir(parents=True)
    executable.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    (fake_app / "Contents/Info.plist").write_text("<plist><dict></dict></plist>\n", encoding="utf-8")

    run(
        [
            "python3",
            str(PREFLIGHT.relative_to(ROOT)),
            "--app",
            str(fake_app.relative_to(ROOT)),
            "--fixtures",
            str(FIXTURES.relative_to(ROOT)),
            "--output",
            str(REPORT.relative_to(ROOT)),
        ],
        failures,
    )

    report = load_json(REPORT, failures) if REPORT.is_file() else {}
    require(report.get("schema_version") == 1, "preflight report schema_version must be 1", failures)
    require(report.get("app_artifact", {}).get("path", "").endswith("SilicaRAW.app"), "preflight must record app artifact path", failures)
    require(report.get("app_artifact", {}).get("sha256"), "preflight must record app artifact hash", failures)
    require(report.get("host", {}).get("macos_version"), "preflight must record macOS version field", failures)
    require(report.get("fixture_path", "").endswith("fixtures"), "preflight must record fixture path", failures)

    hash_results = report.get("hash_results", [])
    require(hash_results, "preflight must record fixture hash results", failures)
    require(all(item.get("ok") is True for item in hash_results), "all fixture hash results must pass", failures)

    limitations = "\n".join(report.get("known_limitations", []))
    for expected in ["RAW decode", "Metal viewer", "AI tools"]:
        require(expected in limitations, f"preflight known limitations must mention {expected}", failures)

    for ignored_path in [manifest_path, REPORT]:
        result = subprocess.run(
            ["git", "check-ignore", str(ignored_path.relative_to(ROOT))],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        require(result.returncode == 0, f"{ignored_path.relative_to(ROOT)} must be ignored by git", failures)

    if failures:
        for failure in failures:
            print(f"qa fixture check failed: {failure}", file=sys.stderr)
        return 1

    print("qa fixtures and installed-app preflight ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
