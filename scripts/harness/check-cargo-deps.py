#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEPENDENCIES_DOC = ROOT / "docs" / "DEPENDENCIES.md"


def cargo_metadata() -> dict:
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def main() -> int:
    metadata = cargo_metadata()
    packages = metadata.get("packages", [])
    deps = sorted(
        {
            dep["name"]
            for package in packages
            for dep in package.get("dependencies", [])
            if dep.get("source") is not None or dep.get("path") is None
        }
    )

    if not deps:
        print("cargo dependencies ok: workspace packages have no external dependencies")
        return 0

    doc = DEPENDENCIES_DOC.read_text(encoding="utf-8")
    missing = [name for name in deps if f"Name: {name}" not in doc and f"Name: {name.replace('_', '-')}" not in doc]

    if missing:
        print("external Cargo dependencies missing docs/DEPENDENCIES.md entries:")
        for name in missing:
            print(f"- {name}")
        return 1

    print(f"cargo dependencies ok: documented {len(deps)} external dependencies")
    return 0


if __name__ == "__main__":
    sys.exit(main())

