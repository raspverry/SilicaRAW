#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRATCH = ROOT / ".tmp/harness/connected-runtime-smoke"
FIXTURES = SCRATCH / "fixtures"
RUN_OUTPUT = SCRATCH / "run"


def run(command, env=None):
    return subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def main():
    if SCRATCH.exists():
        shutil.rmtree(SCRATCH)
    FIXTURES.mkdir(parents=True)
    RUN_OUTPUT.mkdir(parents=True)

    generator = run(
        [
            "python3",
            "scripts/harness/generate-legal-fixtures.py",
            "--output",
            str(FIXTURES.relative_to(ROOT)),
            "--include-raw-placeholders",
        ]
    )
    if generator.returncode != 0:
        print("connected runtime smoke failed: fixture generation failed", file=sys.stderr)
        print(generator.stdout, file=sys.stderr)
        print(generator.stderr, file=sys.stderr)
        return 1

    env = os.environ.copy()
    env["SILICARAW_RUNTIME_SMOKE_FIXTURES"] = str(FIXTURES)
    env["SILICARAW_RUNTIME_SMOKE_OUTPUT"] = str(RUN_OUTPUT)
    smoke = run(
        [
            "cargo",
            "test",
            "-p",
            "silica-desktop",
            "tests::desktop_connected_runtime_smoke",
            "--",
            "--exact",
            "--nocapture",
        ],
        env=env,
    )
    output = smoke.stdout + smoke.stderr
    if smoke.returncode != 0 or "desktop_connected_runtime_smoke" not in output:
        print("connected runtime smoke failed: expected exact desktop runtime smoke test", file=sys.stderr)
        print(smoke.stdout, file=sys.stderr)
        print(smoke.stderr, file=sys.stderr)
        return 1

    print("connected developer runtime smoke ok; clean-Mac DMG QA remains separate")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
