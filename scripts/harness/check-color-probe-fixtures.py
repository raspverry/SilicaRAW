#!/usr/bin/env python3
"""Run the ignored color probe fixture harness against a local legal manifest."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path, PurePosixPath


ROOT = Path(__file__).resolve().parents[2]
ENV_NAME = "SILICARAW_COLOR_FIXTURE_MANIFEST"
LOWER_HEX = set("0123456789abcdef")
REQUIRED_SUBCLASSES = {
    "srgb_jpeg": {
        "input_profile": "srgb",
        "embedded_icc": True,
        "transform_path": "embedded_icc_to_linear_display_p3_to_srgb",
    },
    "display_p3_jpeg": {
        "input_profile": "display_p3",
        "embedded_icc": True,
        "transform_path": "embedded_icc_to_linear_display_p3_to_srgb",
    },
    "untagged_jpeg": {
        "input_profile": "none",
        "embedded_icc": False,
        "transform_path": "assume_srgb_to_linear_display_p3_to_srgb",
    },
}


def fail(message: str, code: int = 1) -> int:
    print(message, file=sys.stderr)
    return code


def load_manifest(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def is_safe_relative_path(value: object) -> bool:
    if not isinstance(value, str) or not value:
        return False
    if "\\" in value or "//" in value:
        return False
    if any(part in {"", ".", ".."} for part in value.split("/")):
        return False
    return not PurePosixPath(value).is_absolute()


def is_lower_sha256(value: object) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(char in LOWER_HEX for char in value)


def validate_manifest(manifest_path: Path, manifest: dict) -> list[dict]:
    if manifest.get("schema") != "silica.fixture_manifest":
        raise ValueError("fixture manifest schema must be silica.fixture_manifest")
    if manifest.get("version") != 1:
        raise ValueError("fixture manifest version must be 1")
    if manifest.get("manifest_kind") != "color-fixtures":
        raise ValueError("color probe harness requires manifest_kind color-fixtures")

    fixtures = manifest.get("fixtures")
    if not isinstance(fixtures, list) or not fixtures:
        raise ValueError("fixture manifest must contain fixtures")
    expected_hashes = manifest.get("expected_source_hashes")
    if not isinstance(expected_hashes, dict):
        raise ValueError("fixture manifest missing expected_source_hashes object")

    seen_subclasses = set()
    validated = []
    for index, fixture in enumerate(fixtures):
        if not isinstance(fixture, dict):
            raise ValueError(f"fixture {index} must be an object")

        fixture_id = fixture.get("id") or f"fixture {index}"
        if fixture.get("class") != "F":
            raise ValueError(f"{fixture_id} must be Color Class F")
        if fixture.get("kind") not in {"tagged_raster", "untagged_raster"}:
            raise ValueError(f"{fixture_id} kind must be tagged_raster or untagged_raster")

        relative_path = fixture.get("relative_path")
        if not is_safe_relative_path(relative_path):
            raise ValueError(f"{fixture_id} has unsafe relative_path")

        integrity = fixture.get("integrity") if isinstance(fixture.get("integrity"), dict) else {}
        expected_hash = integrity.get("sha256")
        if not is_lower_sha256(expected_hash):
            raise ValueError(f"{fixture_id} integrity.sha256 must be lowercase SHA-256")
        if expected_hashes.get(relative_path) != expected_hash:
            raise ValueError(f"{fixture_id} expected_source_hashes must match integrity.sha256")

        color = fixture.get("color") if isinstance(fixture.get("color"), dict) else {}
        profile = (
            fixture.get("profile_expectation")
            if isinstance(fixture.get("profile_expectation"), dict)
            else {}
        )
        subclass = color.get("subclass")
        if subclass not in REQUIRED_SUBCLASSES:
            raise ValueError(f"{fixture_id} color.subclass must be a required Class F subclass")
        seen_subclasses.add(subclass)

        expected = REQUIRED_SUBCLASSES[subclass]
        if profile.get("embedded_icc") is not expected["embedded_icc"]:
            raise ValueError(f"{fixture_id} embedded ICC expectation does not match subclass")
        if profile.get("input_profile_expectation") != expected["input_profile"]:
            raise ValueError(f"{fixture_id} input profile expectation does not match subclass")
        if subclass == "untagged_jpeg" and profile.get("untagged_policy") != "assume_srgb":
            raise ValueError(f"{fixture_id} untagged policy must be assume_srgb")
        if profile.get("color_correctness_proven") is True:
            raise ValueError(f"{fixture_id} must not claim color correctness")

        source_path = manifest_path.parent / relative_path
        if not source_path.is_file():
            raise ValueError(f"{fixture_id} source file missing: {source_path}")

        validated.append(
            {
                "fixture_id": fixture_id,
                "relative_path": relative_path,
                "source_path": source_path,
                "subclass": subclass,
                "expected_hash": expected_hash,
                "expected_probe": expected,
            }
        )

    missing = set(REQUIRED_SUBCLASSES) - seen_subclasses
    if missing:
        raise ValueError(f"fixture manifest missing Class F subclasses: {', '.join(sorted(missing))}")

    return validated


def run_probe(source_path: Path) -> dict[str, str]:
    command = [
        "cargo",
        "run",
        "-p",
        "silica-render",
        "--features",
        "color-probe",
        "--example",
        "color_probe_report",
        "--quiet",
        "--",
        source_path.as_posix(),
    ]
    result = subprocess.run(command, cwd=ROOT, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(
            f"color probe command failed for {source_path}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )

    parsed = {}
    for line in result.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        parsed[key] = value
    return parsed


def verify_probe(fixture: dict, probe: dict[str, str]) -> None:
    expected = fixture["expected_probe"]
    fixture_id = fixture["fixture_id"]

    checks = {
        "status": "success",
        "source_sha256": fixture["expected_hash"],
        "input_profile": expected["input_profile"],
        "embedded_icc": str(expected["embedded_icc"]).lower(),
        "working_space": "linear_display_p3",
        "output_profile": "srgb",
        "transform_path": expected["transform_path"],
        "error_category": "none",
    }
    for key, value in checks.items():
        if probe.get(key) != value:
            raise ValueError(f"{fixture_id} probe {key} expected {value}, got {probe.get(key)}")


def main() -> int:
    manifest_value = os.environ.get(ENV_NAME)
    if not manifest_value:
        return fail(f"{ENV_NAME} must point to a legal color fixture manifest", 2)

    manifest_path = Path(manifest_value)
    if not manifest_path.is_absolute() or not manifest_path.is_file():
        return fail(f"{ENV_NAME} must be an absolute path to a manifest file", 2)

    try:
        manifest = load_manifest(manifest_path)
        fixtures = validate_manifest(manifest_path, manifest)
    except Exception as exc:
        return fail(f"invalid color fixture manifest: {exc}", 2)

    report = {
        "manifest_path": manifest_path.as_posix(),
        "fixture_count": len(fixtures),
        "results": [],
    }
    failures = []

    for fixture in fixtures:
        source_path = fixture["source_path"]
        try:
            before_hash = sha256_file(source_path)
            before_stat = source_path.stat()
            if before_hash != fixture["expected_hash"]:
                raise ValueError(
                    f"{fixture['fixture_id']} source hash mismatch before probe: {before_hash}"
                )

            probe = run_probe(source_path)
            verify_probe(fixture, probe)

            after_hash = sha256_file(source_path)
            after_stat = source_path.stat()
            hash_unchanged = before_hash == after_hash
            size_unchanged = before_stat.st_size == after_stat.st_size
            modified_time_unchanged = before_stat.st_mtime_ns == after_stat.st_mtime_ns
            if not hash_unchanged:
                raise ValueError(f"{fixture['fixture_id']} source hash changed after probe")

            report["results"].append(
                {
                    "fixture_id": fixture["fixture_id"],
                    "relative_path": fixture["relative_path"],
                    "subclass": fixture["subclass"],
                    "source_sha256": after_hash,
                    "file_size": after_stat.st_size,
                    "modified_time_ns": after_stat.st_mtime_ns,
                    "original_hash_unchanged": hash_unchanged,
                    "original_size_unchanged": size_unchanged,
                    "original_modified_time_unchanged": modified_time_unchanged,
                    "platform": probe.get("platform"),
                    "status": probe.get("status"),
                    "input_profile": probe.get("input_profile"),
                    "embedded_icc": probe.get("embedded_icc") == "true",
                    "working_space": probe.get("working_space"),
                    "output_profile": probe.get("output_profile"),
                    "transform_path": probe.get("transform_path"),
                    "error_category": probe.get("error_category"),
                }
            )
        except Exception as exc:
            failures.append(str(exc))

    print(json.dumps(report, indent=2, sort_keys=True))
    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
