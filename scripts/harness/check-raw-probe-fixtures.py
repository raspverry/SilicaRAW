#!/usr/bin/env python3
"""Run the ignored RAW probe fixture harness against a local legal manifest."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    manifest = os.environ.get("SILICARAW_RAW_FIXTURE_MANIFEST")
    if not manifest:
        print(
            "SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest",
            file=sys.stderr,
        )
        return 2

    manifest_path = Path(manifest)
    if not manifest_path.is_absolute() or not manifest_path.is_file():
        print(
            "SILICARAW_RAW_FIXTURE_MANIFEST must be an absolute path to a manifest file",
            file=sys.stderr,
        )
        return 2

    command = [
        "cargo",
        "test",
        "-p",
        "silica-decode",
        "--features",
        "core-image-raw-probe",
        "probes_raw_fixture_manifest_without_mutating_originals",
        "--",
        "--ignored",
        "--exact",
    ]
    return subprocess.run(command, cwd=ROOT, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
