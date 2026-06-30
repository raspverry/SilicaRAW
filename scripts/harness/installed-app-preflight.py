#!/usr/bin/env python3
import argparse
import hashlib
import json
import platform
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


KNOWN_LIMITATIONS = [
    "RAW decode is blocked in the local alpha; RAW placeholder files must remain non-decodable.",
    "Metal viewer output is disabled in the local alpha; previews use the current standard raster runtime path.",
    "AI tools are disabled in the local alpha; no MLX runtime, model loading, or MCP tooling is exercised.",
    "The developer preflight records unsigned/local artifact evidence only; clean-Mac DMG QA remains a later gate.",
]


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


def load_manifest(fixtures):
    manifest_path = fixtures / "fixture-manifest.json"
    if not manifest_path.is_file():
        raise FileNotFoundError(f"missing fixture manifest: {manifest_path}")
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def verify_fixture_hashes(fixtures, manifest):
    expected_hashes = manifest.get("expected_source_hashes", {})
    results = []
    for fixture in manifest.get("fixtures", []):
        relative_path = fixture.get("relative_path")
        if not relative_path:
            continue
        path = fixtures / relative_path
        expected = expected_hashes.get(relative_path) or fixture.get("sha256")
        actual = sha256_file(path) if path.is_file() else None
        results.append(
            {
                "relative_path": relative_path,
                "expected_sha256": expected,
                "actual_sha256": actual,
                "ok": actual == expected,
                "role": fixture.get("role"),
                "preview_status": fixture.get("expected", {}).get("preview_status"),
            }
        )
    return results


def host_record():
    macos_version = platform.mac_ver()[0] or "not-macos"
    return {
        "system": platform.system(),
        "machine": platform.machine(),
        "platform": platform.platform(),
        "macos_version": macos_version,
    }


def parse_args():
    parser = argparse.ArgumentParser(description="Record installed-app developer preflight evidence.")
    parser.add_argument("--app", type=Path, required=True, help="Path to the local SilicaRAW .app or app artifact.")
    parser.add_argument("--fixtures", type=Path, required=True, help="Path to generated legal QA fixtures.")
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / ".tmp/installed-app-preflight/installed-app-preflight.json",
        help="JSON report output path. Defaults to .tmp/installed-app-preflight/installed-app-preflight.json.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    app = args.app if args.app.is_absolute() else ROOT / args.app
    fixtures = args.fixtures if args.fixtures.is_absolute() else ROOT / args.fixtures
    output = args.output if args.output.is_absolute() else ROOT / args.output

    if not app.exists():
        print(f"app artifact does not exist: {app}", file=sys.stderr)
        return 1
    if not fixtures.is_dir():
        print(f"fixture path does not exist or is not a directory: {fixtures}", file=sys.stderr)
        return 1

    try:
        manifest = load_manifest(fixtures)
        hash_results = verify_fixture_hashes(fixtures, manifest)
        report = {
            "schema_version": 1,
            "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
            "preflight": "developer-installed-app",
            "app_artifact": artifact_record(app),
            "host": host_record(),
            "fixture_path": fixtures.as_posix(),
            "fixture_manifest": {
                "path": (fixtures / "fixture-manifest.json").as_posix(),
                "schema_version": manifest.get("schema_version"),
                "fixture_count": len(manifest.get("fixtures", [])),
                "include_raw_placeholders": manifest.get("include_raw_placeholders", False),
                "source_policy": manifest.get("source_policy"),
            },
            "hash_results": hash_results,
            "known_limitations": KNOWN_LIMITATIONS,
        }
    except Exception as exc:
        print(f"failed to build installed-app preflight report: {exc}", file=sys.stderr)
        return 1

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if not hash_results or any(not item.get("ok") for item in hash_results):
        print(f"preflight hash verification failed; report written to {output}", file=sys.stderr)
        return 1

    print(f"installed-app preflight report written to {output.relative_to(ROOT) if output.is_relative_to(ROOT) else output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
