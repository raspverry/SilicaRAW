#!/usr/bin/env python3
import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import sys
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


def parse_attach_mount(stdout):
    for line in stdout.splitlines():
        parts = [part for part in line.split("\t") if part]
        if parts and parts[-1].startswith("/Volumes/"):
            return Path(parts[-1])
    return None


def parse_args():
    parser = argparse.ArgumentParser(description="Verify a local SilicaRAW DMG artifact on the build Mac.")
    parser.add_argument("--dmg", type=Path, required=True, help="Path to SilicaRAW DMG artifact.")
    parser.add_argument(
        "--installed-app",
        type=Path,
        help="Optional installed app path to compare against the mounted DMG app, for example /Applications/SilicaRAW.app.",
    )
    parser.add_argument(
        "--install-to",
        type=Path,
        help="Optional empty destination path to copy SilicaRAW.app into. Existing destinations are rejected.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / ".tmp/local-dmg-smoke/local-dmg-artifact-smoke.json",
        help="JSON report output path.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    dmg = args.dmg if args.dmg.is_absolute() else ROOT / args.dmg
    output = args.output if args.output.is_absolute() else ROOT / args.output
    installed_app = args.installed_app
    install_to = args.install_to
    if installed_app and not installed_app.is_absolute():
        installed_app = ROOT / installed_app
    if install_to and not install_to.is_absolute():
        install_to = ROOT / install_to

    failures = []
    if not dmg.is_file():
        print(f"DMG does not exist: {dmg}", file=sys.stderr)
        return 1

    verify = run(["hdiutil", "verify", dmg.as_posix()])
    if verify.returncode != 0:
        failures.append({"step": "hdiutil verify", "stdout": verify.stdout, "stderr": verify.stderr})

    mount_point = None
    mounted_app = None
    attach = run(["hdiutil", "attach", dmg.as_posix(), "-nobrowse", "-readonly"])
    if attach.returncode != 0:
        failures.append({"step": "hdiutil attach", "stdout": attach.stdout, "stderr": attach.stderr})
    else:
        mount_point = parse_attach_mount(attach.stdout)
        if not mount_point:
            failures.append({"step": "parse mount point", "stdout": attach.stdout, "stderr": attach.stderr})
        else:
            mounted_app = mount_point / "SilicaRAW.app"
            if not mounted_app.is_dir():
                failures.append({"step": "mounted app exists", "path": mounted_app.as_posix()})

    copied = False
    try:
        if mounted_app and mounted_app.is_dir() and install_to:
            if install_to.exists():
                failures.append({"step": "install destination empty", "path": install_to.as_posix()})
            else:
                shutil.copytree(mounted_app, install_to, symlinks=True)
                copied = True
                installed_app = install_to

        installed_matches_mounted = None
        if mounted_app and mounted_app.is_dir() and installed_app:
            if not installed_app.is_dir():
                failures.append({"step": "installed app exists", "path": installed_app.as_posix()})
            else:
                installed_matches_mounted = sha256_tree(mounted_app)[0] == sha256_tree(installed_app)[0]

        report = {
            "schema_version": 1,
            "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
            "smoke": "local-build-mac-dmg-artifact",
            "host": host_record(),
            "dmg": artifact_record(dmg),
            "hdiutil_verify_ok": verify.returncode == 0,
            "mount_point": mount_point.as_posix() if mount_point else None,
            "mounted_app": artifact_record(mounted_app) if mounted_app and mounted_app.is_dir() else None,
            "installed_app": artifact_record(installed_app) if installed_app and installed_app.is_dir() else None,
            "installed_matches_mounted": installed_matches_mounted,
            "copied_to_install_path": copied,
            "clean_mac_gate_remains": True,
            "failures": failures,
        }
    finally:
        if mount_point:
            detach = run(["hdiutil", "detach", mount_point.as_posix()])
            if detach.returncode != 0:
                failures.append({"step": "hdiutil detach", "stdout": detach.stdout, "stderr": detach.stderr})

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if failures:
        print(f"local DMG artifact smoke failed; report written to {output}", file=sys.stderr)
        return 1

    print(f"local DMG artifact smoke report written to {output.relative_to(ROOT) if output.is_relative_to(ROOT) else output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
