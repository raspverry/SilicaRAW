#!/usr/bin/env python3
import argparse
import hashlib
import json
import platform
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


REQUIRED_SECRET_NAMES = [
    "APPLE_CERTIFICATE",
    "APPLE_CERTIFICATE_PASSWORD",
    "KEYCHAIN_PASSWORD",
    "APPLE_ID",
    "APPLE_PASSWORD",
    "APPLE_TEAM_ID",
]

REQUIRED_SECRET_OR_VARIABLE_NAMES = [
    "APPLE_SIGNING_IDENTITY",
]


def run(command):
    return subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)


def host_record():
    return {
        "system": platform.system(),
        "machine": platform.machine(),
        "platform": platform.platform(),
        "macos_version": platform.mac_ver()[0] or "not-macos",
    }


def sha256_text(value):
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def parse_code_signing_identities(output):
    identities = []
    for line in output.splitlines():
        match = re.search(r'^\s*\d+\)\s+([0-9A-Fa-f]+)\s+"([^"]+)"', line)
        if match:
            identities.append(
                {
                    "hash_prefix": match.group(1)[:12],
                    "name": match.group(2),
                    "name_sha256": sha256_text(match.group(2)),
                }
            )
    return identities


def parse_gh_names(output):
    names = set()
    for line in output.splitlines():
        if line.strip():
            names.add(line.split()[0])
    return names


def parse_args():
    parser = argparse.ArgumentParser(
        description="Check local and GitHub prerequisites for signed/notarized macOS DMG release work."
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / ".tmp/signing-prereqs/signing-prereqs.json",
        help="JSON report output path.",
    )
    parser.add_argument(
        "--fail-on-missing",
        action="store_true",
        help="Return a non-zero exit code when signing prerequisites are missing.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    output = args.output if args.output.is_absolute() else ROOT / args.output

    security = run(["security", "find-identity", "-v", "-p", "codesigning"])
    identities = parse_code_signing_identities(security.stdout)
    developer_id_identities = [
        identity for identity in identities if identity["name"].startswith("Developer ID Application:")
    ]

    gh_secrets = run(["gh", "secret", "list"])
    secret_names = parse_gh_names(gh_secrets.stdout) if gh_secrets.returncode == 0 else set()

    gh_variables = run(["gh", "variable", "list"])
    variable_names = parse_gh_names(gh_variables.stdout) if gh_variables.returncode == 0 else set()

    missing_secrets = [name for name in REQUIRED_SECRET_NAMES if name not in secret_names]
    missing_secret_or_variable = [
        name for name in REQUIRED_SECRET_OR_VARIABLE_NAMES if name not in secret_names and name not in variable_names
    ]
    failures = []
    if security.returncode != 0:
        failures.append({"step": "security find-identity", "stderr": security.stderr})
    if gh_secrets.returncode != 0:
        failures.append({"step": "gh secret list", "stderr": gh_secrets.stderr})
    if gh_variables.returncode != 0:
        failures.append({"step": "gh variable list", "stderr": gh_variables.stderr})

    report = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "preflight": "macos-signing-notarization-prereqs",
        "host": host_record(),
        "local_code_signing_identities": identities,
        "developer_id_application_identities": developer_id_identities,
        "developer_id_application_present": bool(developer_id_identities),
        "github_secret_names_present": sorted(secret_names),
        "github_variable_names_present": sorted(variable_names),
        "required_secret_names": REQUIRED_SECRET_NAMES,
        "required_secret_or_variable_names": REQUIRED_SECRET_OR_VARIABLE_NAMES,
        "missing_secret_names": missing_secrets,
        "missing_secret_or_variable_names": missing_secret_or_variable,
        "ready_for_task_7_1": bool(developer_id_identities)
        and not missing_secrets
        and not missing_secret_or_variable
        and not failures,
        "failures": failures,
        "notes": [
            "This report records secret and variable names only; it never reads secret values.",
            "APPLE_PASSWORD is the Apple ID app-specific password for the documented Tauri notarization path.",
            "App Store Connect API notarization can be added later as an explicit alternative path.",
        ],
    }

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    missing = missing_secrets or missing_secret_or_variable or not developer_id_identities or failures
    if missing:
        print(f"signing prereqs incomplete; report written to {output}", file=sys.stderr)
        return 1 if args.fail_on_missing else 0

    print(f"signing prereqs ready; report written to {output.relative_to(ROOT) if output.is_relative_to(ROOT) else output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
