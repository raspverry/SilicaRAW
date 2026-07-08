#!/usr/bin/env python3
import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def run(command):
    return subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)


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
        "path": path.as_posix(),
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


def list_app_processes(app):
    expected_prefix = (app / "Contents/MacOS/").as_posix()
    ps = run(["ps", "-axo", "pid,command"])
    if ps.returncode != 0:
        return []
    processes = []
    for line in ps.stdout.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        parts = stripped.split(maxsplit=1)
        if len(parts) != 2:
            continue
        pid_text, command = parts
        if command.startswith(expected_prefix):
            processes.append({"pid": int(pid_text), "command": command})
    return processes


def find_new_app_process(app, before_pids):
    for process in list_app_processes(app):
        if process["pid"] not in before_pids:
            return process
    return None


def codesign_record(app):
    result = run(["codesign", "-dv", "--verbose=4", app.as_posix()])
    output = result.stdout + result.stderr
    record = {
        "returncode": result.returncode,
        "signature": None,
        "team_identifier": None,
        "format": None,
        "raw_tail": output.splitlines()[-40:],
    }
    for line in output.splitlines():
        if line.startswith("Signature="):
            record["signature"] = line.split("=", 1)[1]
        elif line.startswith("TeamIdentifier="):
            record["team_identifier"] = line.split("=", 1)[1]
        elif line.startswith("Format="):
            record["format"] = line.split("=", 1)[1]
    return record


def parse_args():
    parser = argparse.ArgumentParser(description="Launch an installed SilicaRAW.app and record its process path.")
    parser.add_argument(
        "--app",
        type=Path,
        default=Path("/Applications/SilicaRAW.app"),
        help="Installed app bundle to launch.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / ".tmp/q6-installed-app-launch/installed-app-launch-smoke.json",
        help="JSON report output path.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=10.0,
        help="Seconds to wait for the app process.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    app = args.app if args.app.is_absolute() else ROOT / args.app
    output = args.output if args.output.is_absolute() else ROOT / args.output
    executable = app / "Contents/MacOS/silica-desktop"
    failures = []

    if not app.is_dir():
        print(f"app does not exist: {app}", file=sys.stderr)
        return 1
    if not executable.is_file():
        print(f"app executable does not exist: {executable}", file=sys.stderr)
        return 1

    before_processes = list_app_processes(app)
    before_pids = {process["pid"] for process in before_processes}

    launch = run(["open", "-n", app.as_posix()])
    if launch.returncode != 0:
        failures.append({"step": "open", "stdout": launch.stdout, "stderr": launch.stderr})

    process = None
    deadline = time.monotonic() + args.timeout_seconds
    while time.monotonic() < deadline and not process:
        process = find_new_app_process(app, before_pids)
        if not process:
            time.sleep(0.25)

    if not process:
        failures.append({"step": "find app process", "app": app.as_posix()})

    stopped = None
    if process:
        kill = run(["kill", str(process["pid"])])
        if kill.returncode != 0:
            failures.append({"step": "kill app process", "stdout": kill.stdout, "stderr": kill.stderr})
            stopped = False
        else:
            time.sleep(1.0)
            stopped = all(item["pid"] != process["pid"] for item in list_app_processes(app))
            if not stopped:
                failures.append({"step": "confirm app process stopped", "pid": process["pid"]})

    app_path = app.as_posix()
    report = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "smoke": "installed-app-launch",
        "host": host_record(),
        "app": artifact_record(app),
        "executable": artifact_record(executable),
        "codesign": codesign_record(app),
        "launch_command": ["open", "-n", app_path],
        "preexisting_processes": before_processes,
        "process": process,
        "process_path_matches_app": bool(process and process["command"].startswith(app_path)),
        "launched_from_applications": app_path.startswith("/Applications/"),
        "launched_from_repo_checkout": bool(process and str(ROOT) in process["command"]),
        "process_stopped": stopped,
        "clean_mac_gate_remains": True,
        "workflow_from_installed_app_not_run": True,
        "failures": failures,
    }

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if failures:
        print(f"installed app launch smoke failed; report written to {output}", file=sys.stderr)
        return 1

    print(f"installed app launch smoke report written to {output.relative_to(ROOT) if output.is_relative_to(ROOT) else output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
