#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import platform
import shutil
import sqlite3
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_APP = Path("/Applications/SilicaRAW.app")
DEFAULT_SCRATCH = ROOT / ".tmp/q6-installed-workflow"
GENERATOR = ROOT / "scripts/harness/generate-legal-fixtures.py"

IMPORT_COPY_PLAN = [
    {
        "source": "supported/reference-urban.jpg",
        "destination": "reference-urban.jpg",
        "role": "supported-jpeg",
    },
    {
        "source": "supported/reference-still-life.jpeg",
        "destination": "reference-still-life.jpeg",
        "role": "supported-jpeg",
    },
    {
        "source": "raw-blocked/blocked-raw.DNG",
        "destination": "blocked-raw.DNG",
        "role": "raw-blocked-placeholder",
    },
    {
        "source": "unsupported/notes.txt",
        "destination": "notes.txt",
        "role": "unsupported",
    },
]


def relative_path(path):
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def run(command, env=None):
    return subprocess.run(command, cwd=ROOT, env=env, text=True, capture_output=True, check=False)


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def partial_file_hash(path):
    remaining = 64 * 1024
    hash_value = 0xCBF29CE484222325
    with path.open("rb") as handle:
        while remaining > 0:
            chunk = handle.read(min(8192, remaining))
            if not chunk:
                break
            for byte in chunk:
                hash_value ^= byte
                hash_value = (hash_value * 0x00000100000001B3) & 0xFFFFFFFFFFFFFFFF
            remaining -= len(chunk)
    return f"{hash_value:016x}"


def file_stat_record(path):
    stat = path.stat()
    return {
        "size_bytes": stat.st_size,
        "mtime_ns": stat.st_mtime_ns,
        "modified_at": f"unix:{int(stat.st_mtime)}",
        "partial_hash": partial_file_hash(path),
        "sha256": sha256_file(path),
    }


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


def git_commit():
    result = run(["git", "rev-parse", "HEAD"])
    return result.stdout.strip() if result.returncode == 0 else None


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


def seed_import_originals(fixtures, import_root):
    records = []
    for item in IMPORT_COPY_PLAN:
        source = fixtures / item["source"]
        destination = import_root / item["destination"]
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        before = file_stat_record(destination)
        records.append(
            {
                "source_fixture": item["source"],
                "original_path": destination.as_posix(),
                "role": item["role"],
                "before_size_bytes": before["size_bytes"],
                "before_mtime_ns": before["mtime_ns"],
                "before_modified_at": before["modified_at"],
                "before_partial_hash": before["partial_hash"],
                "before_sha256": before["sha256"],
            }
        )
    return records


def finalize_original_hashes(records):
    finalized = []
    for record in records:
        path = Path(record["original_path"])
        after = file_stat_record(path) if path.is_file() else {}
        after_sha256 = after.get("sha256")
        finalized.append(
            {
                **record,
                "after_size_bytes": after.get("size_bytes"),
                "after_mtime_ns": after.get("mtime_ns"),
                "after_modified_at": after.get("modified_at"),
                "after_partial_hash": after.get("partial_hash"),
                "after_sha256": after_sha256,
                "hash_ok": after_sha256 == record["before_sha256"],
                "size_ok": after.get("size_bytes") == record["before_size_bytes"],
                "partial_hash_ok": after.get("partial_hash") == record["before_partial_hash"],
                "ok": after_sha256 == record["before_sha256"]
                and after.get("size_bytes") == record["before_size_bytes"]
                and after.get("partial_hash") == record["before_partial_hash"],
            }
        )
    return finalized


def path_is_relative_to(path, parent):
    try:
        path.relative_to(parent)
        return True
    except ValueError:
        return False


def query_rows(catalog_db, sql, params=()):
    if not catalog_db.is_file():
        return []
    connection = sqlite3.connect(catalog_db)
    connection.row_factory = sqlite3.Row
    try:
        rows = connection.execute(sql, params).fetchall()
        return [dict(row) for row in rows]
    finally:
        connection.close()


def query_scalar(catalog_db, sql, params=()):
    rows = query_rows(catalog_db, sql, params)
    if not rows:
        return None
    return next(iter(rows[0].values()))


def catalog_rows(catalog_db, library_root, import_root):
    rows = query_rows(
        catalog_db,
        """
        SELECT
          p.id,
          p.path,
          p.file_name,
          p.file_size,
          p.modified_at,
          p.partial_hash,
          p.full_hash,
          p.missing,
          p.unsupported,
          p.file_type,
          pf.rating,
          pf.picked,
          pf.rejected,
          pf.color_label,
          pf.edited,
          pf.exported
        FROM photos p
        LEFT JOIN photo_flags pf ON pf.photo_id = p.id
        ORDER BY p.path
        """,
    )
    result = []
    for row in rows:
        source_path = Path(row["path"])
        source_stat = file_stat_record(source_path) if source_path.is_file() else {}
        result.append(
            {
                **row,
                "missing": bool(row["missing"]),
                "unsupported": bool(row["unsupported"]),
                "picked": bool(row["picked"]) if row["picked"] is not None else None,
                "rejected": bool(row["rejected"]) if row["rejected"] is not None else None,
                "edited": bool(row["edited"]) if row["edited"] is not None else None,
                "exported": bool(row["exported"]) if row["exported"] is not None else None,
                "source_exists": source_path.is_file(),
                "references_import_root": path_is_relative_to(source_path, import_root),
                "outside_library_root": not path_is_relative_to(source_path, library_root),
                "current_source": source_stat,
                "file_size_matches_source": row["file_size"] == source_stat.get("size_bytes"),
                "modified_at_matches_source": row["modified_at"] == source_stat.get("modified_at"),
                "partial_hash_matches_source": row["partial_hash"] == source_stat.get("partial_hash"),
                "full_hash_matches_source": row["full_hash"] == source_stat.get("sha256"),
            }
        )
    return result


def active_edit_states(catalog_db):
    rows = query_rows(
        catalog_db,
        """
        SELECT id, photo_id, active, edit_graph_json
        FROM edit_states
        WHERE active = 1
        ORDER BY id
        """,
    )
    result = []
    for row in rows:
        graph = json.loads(row["edit_graph_json"])
        basic = graph.get("basic", {})
        result.append(
            {
                "id": row["id"],
                "photo_id": row["photo_id"],
                "active": bool(row["active"]),
                "exposure": basic.get("exposure"),
                "contrast": basic.get("contrast"),
                "source": graph.get("source", {}),
            }
        )
    return result


def export_rows(catalog_db):
    rows = query_rows(
        catalog_db,
        """
        SELECT e.id, e.photo_id, e.output_path, e.export_settings_json,
               p.path AS source_path, p.full_hash AS source_full_hash
        FROM exports e
        JOIN photos p ON p.id = e.photo_id
        ORDER BY e.id
        """,
    )
    result = []
    for row in rows:
        settings = json.loads(row["export_settings_json"])
        result.append(
            {
                "id": row["id"],
                "photo_id": row["photo_id"],
                "output_path": row["output_path"],
                "source_path": row["source_path"],
                "settings": {
                    "format": settings.get("format"),
                    "color_profile": settings.get("color_profile"),
                    "quality": settings.get("quality"),
                    "source_path": settings.get("source_path"),
                    "source_sha256": settings.get("source_sha256"),
                    "source_sha256_after_export": settings.get("source_sha256_after_export"),
                    "source_original_hash_unchanged": settings.get("source_original_hash_unchanged"),
                    "output_sha256": settings.get("output_sha256"),
                },
                "source_path_matches_photo": settings.get("source_path") == row["source_path"],
                "source_sha_matches_photo": settings.get("source_sha256") == row["source_full_hash"],
                "source_sha_after_matches_before": settings.get("source_sha256_after_export")
                == settings.get("source_sha256"),
                "source_original_hash_unchanged": settings.get("source_original_hash_unchanged")
                is True,
                "output_exists": Path(row["output_path"]).is_file(),
            }
        )
    return result


def marker_record(marker_path):
    if not marker_path.is_file():
        return {"exists": False, "fields": {}}
    fields = {}
    for line in marker_path.read_text(encoding="utf-8").splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            fields[key] = value
    return {
        "exists": True,
        "path": marker_path.as_posix(),
        "fields": fields,
        "raw": marker_path.read_text(encoding="utf-8"),
    }


def sips_record(output_path):
    result = run(
        [
            "sips",
            "-g",
            "format",
            "-g",
            "pixelWidth",
            "-g",
            "pixelHeight",
            "-g",
            "space",
            output_path.as_posix(),
        ]
    )
    properties = {}
    for line in result.stdout.splitlines():
        stripped = line.strip()
        if ": " in stripped:
            key, value = stripped.split(": ", 1)
            properties[key] = value
    return {
        "returncode": result.returncode,
        "properties": properties,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def command_tail(output, limit=80):
    return output.splitlines()[-limit:]


def parse_args():
    parser = argparse.ArgumentParser(
        description="Record Q6.2 installed app workflow evidence from /Applications/SilicaRAW.app."
    )
    parser.add_argument(
        "--app",
        type=Path,
        default=DEFAULT_APP,
        help="Installed app bundle to execute.",
    )
    parser.add_argument(
        "--scratch",
        type=Path,
        default=DEFAULT_SCRATCH,
        help="Scratch directory for fixtures and workflow output.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_SCRATCH / "installed-app-workflow-evidence.json",
        help="JSON evidence report output path.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    app = args.app if args.app.is_absolute() else ROOT / args.app
    scratch = args.scratch if args.scratch.is_absolute() else ROOT / args.scratch
    output = args.output if args.output.is_absolute() else ROOT / args.output
    executable = app / "Contents/MacOS/silica-desktop"
    fixtures = scratch / "fixtures"
    run_output = scratch / "run"
    library_root = run_output / "SilicaRAW Library"
    import_root = run_output / "Import Originals"
    export_root = run_output / "Exports"
    catalog_db = library_root / "catalog.db"
    marker_path = run_output / "installed-workflow-smoke.marker"
    export_output = export_root / "reference-urban-export.jpg"

    failures = []
    if not app.is_dir():
        print(f"installed app does not exist: {app}", file=sys.stderr)
        return 1
    if not executable.is_file():
        print(f"installed app executable does not exist: {executable}", file=sys.stderr)
        return 1

    if scratch.exists():
        shutil.rmtree(scratch)
    fixtures.mkdir(parents=True)
    run_output.mkdir(parents=True)
    import_root.mkdir(parents=True)
    export_root.mkdir(parents=True)

    generator = generate_fixtures(fixtures)
    if generator.returncode != 0:
        print("installed app workflow evidence failed: fixture generation failed", file=sys.stderr)
        print(generator.stdout, file=sys.stderr)
        print(generator.stderr, file=sys.stderr)
        return 1

    manifest = load_json(fixtures / "fixture-manifest.json")
    before_originals = seed_import_originals(fixtures, import_root)

    env = {
        **os.environ,
        "SILICARAW_INSTALLED_WORKFLOW_SMOKE": "1",
        "SILICARAW_INSTALLED_WORKFLOW_FIXTURES": fixtures.as_posix(),
        "SILICARAW_INSTALLED_WORKFLOW_OUTPUT": run_output.as_posix(),
    }
    command = [executable.as_posix()]
    smoke = run(command, env=env)
    smoke_output = smoke.stdout + smoke.stderr
    marker = marker_record(marker_path)
    rows = catalog_rows(catalog_db, library_root, import_root)
    edits = active_edit_states(catalog_db)
    exports = export_rows(catalog_db)
    original_hashes = finalize_original_hashes(before_originals)
    sips = sips_record(export_output) if export_output.is_file() else {"returncode": None}
    primary_row = next((row for row in rows if row["file_name"] == "reference-urban.jpg"), {})
    marker_exe = marker.get("fields", {}).get("current_exe")

    checks = {
        "app_path_is_applications": app.as_posix().startswith("/Applications/"),
        "executable_path_is_installed_app": executable.as_posix().startswith(app.as_posix()),
        "installed_workflow_command_passed": smoke.returncode == 0,
        "installed_workflow_completion_marker_present": "installed app workflow smoke complete"
        in smoke_output,
        "marker_file_present": marker.get("exists") is True,
        "marker_current_exe_is_installed_app": bool(marker_exe)
        and marker_exe.startswith(app.as_posix())
        and str(ROOT) not in marker_exe,
        "catalog_db_exists": catalog_db.is_file(),
        "schema_version_at_least_12": (query_scalar(catalog_db, "SELECT COALESCE(MAX(version), 0) FROM schema_migrations") or 0)
        >= 12,
        "catalog_rows_present": len(rows) == 4,
        "catalog_paths_reference_import_root": bool(rows)
        and all(row["references_import_root"] for row in rows),
        "catalog_paths_outside_library_root": bool(rows)
        and all(row["outside_library_root"] for row in rows),
        "catalog_fingerprints_match_sources": bool(rows)
        and all(
            row["file_size_matches_source"]
            and row["modified_at_matches_source"]
            and row["partial_hash_matches_source"]
            and row["full_hash_matches_source"]
            for row in rows
        ),
        "primary_flags_persisted": primary_row.get("rating") == 4
        and primary_row.get("picked") is True
        and primary_row.get("rejected") is False
        and primary_row.get("color_label") == "green"
        and primary_row.get("edited") is True
        and primary_row.get("exported") is True,
        "active_edit_persisted": bool(edits)
        and any(edit["exposure"] == 0.4 and edit["contrast"] == 12.0 for edit in edits),
        "jpeg_srgb_export_recorded": bool(exports)
        and all(
            export["source_path_matches_photo"]
            and export["source_sha_matches_photo"]
            and export["source_sha_after_matches_before"]
            and export["source_original_hash_unchanged"]
            and export["output_exists"]
            and export["settings"]["format"] == "jpeg"
            and export["settings"]["color_profile"] == "srgb"
            for export in exports
        ),
        "export_output_opens_with_sips": sips.get("returncode") == 0
        and sips.get("properties", {}).get("format") == "jpeg",
        "original_hashes_unchanged": bool(original_hashes)
        and all(record["ok"] for record in original_hashes),
    }
    failures.extend(check for check, passed in checks.items() if not passed)

    report = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "smoke": "q6-installed-app-workflow",
        "source_commit": git_commit(),
        "host": host_record(),
        "app": artifact_record(app),
        "executable": artifact_record(executable),
        "command": command,
        "run_output": relative_path(run_output),
        "library_path": relative_path(library_root),
        "import_path": relative_path(import_root),
        "export_path": relative_path(export_root),
        "fixture_manifest": {
            "path": relative_path(fixtures / "fixture-manifest.json"),
            "schema_version": manifest.get("schema_version"),
            "fixture_count": len(manifest.get("fixtures", [])),
            "include_raw_placeholders": manifest.get("include_raw_placeholders", False),
            "source_policy": manifest.get("source_policy"),
        },
        "installed_workflow_boundary": "The installed app executable ran the workflow smoke. This is not WebView click, native path picker, Gatekeeper, notarization, GitHub Release download, offline, or clean-Mac evidence.",
        "marker": marker,
        "catalog": {
            "db_path": relative_path(catalog_db),
            "schema_version": query_scalar(
                catalog_db, "SELECT COALESCE(MAX(version), 0) FROM schema_migrations"
            ),
            "row_count": len(rows),
            "rows": rows,
            "active_edit_states": edits,
            "exports": exports,
        },
        "sips": sips,
        "original_hashes": original_hashes,
        "checks": checks,
        "installed_workflow_smoke": {
            "returncode": smoke.returncode,
            "stdout_stderr_tail": command_tail(smoke_output),
        },
        "known_limitations": [
            "This evidence runs the installed app executable from /Applications.",
            "It does not automate WebView clicks, native path picker, or menu interactions.",
            "It does not prove offline behavior, Gatekeeper acceptance, signed/notarized behavior, GitHub Release download behavior, or clean-Mac behavior.",
        ],
        "failures": failures,
    }

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if failures:
        print(f"installed app workflow evidence failed; report written to {relative_path(output)}", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(f"installed app workflow evidence report written to {relative_path(output)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
