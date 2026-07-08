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
DEFAULT_SCRATCH = ROOT / ".tmp/q5-library-import-reference"
DEFAULT_APP = ROOT / "target/release/bundle/macos/SilicaRAW.app"
GENERATOR = ROOT / "scripts/harness/generate-legal-fixtures.py"


KNOWN_LIMITATIONS = [
    "This evidence records a developer app artifact and exercises the same desktop command boundary through the Rust connected runtime smoke.",
    "It does not automate the WebView GUI, native path picker, menu commands, or drag-to-/Applications install.",
    "It does not prove DMG mount behavior, Gatekeeper acceptance, signed/notarized behavior, offline behavior, or clean-Mac downloaded-artifact behavior.",
]


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
    {
        "source": "supported/reference-still-life.jpeg",
        "destination": "Recursive/recursive-child.jpg",
        "role": "supported-jpeg",
    },
    {
        "source": "unsupported/notes.txt",
        "destination": "Recursive/recursive-notes.txt",
        "role": "unsupported",
    },
    {
        "source": "supported/reference-urban.jpg",
        "destination": "Recursive/.hidden.jpg",
        "role": "hidden-skipped",
    },
    {
        "source": "supported/reference-urban.jpg",
        "destination": "Recursive/Archive.photoslibrary/package-child.jpg",
        "role": "package-child-skipped",
    },
]


def relative_path(path):
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


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
        "path": relative_path(path),
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


def run(command, env=None):
    return subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


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
                "original_path": relative_path(destination),
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
        path = ROOT / record["original_path"]
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


def schema_version(catalog_db):
    return query_scalar(catalog_db, "SELECT COALESCE(MAX(version), 0) FROM schema_migrations")


def library_rows(catalog_db):
    return query_rows(
        catalog_db,
        """
        SELECT id, root_path, created_at, updated_at
        FROM libraries
        ORDER BY id
        """,
    )


def folder_rows(catalog_db):
    rows = query_rows(
        catalog_db,
        """
        SELECT id, library_id, path, scanned_at, missing
        FROM folders
        ORDER BY path
        """,
    )
    for row in rows:
        row["missing"] = bool(row["missing"])
    return rows


def catalog_rows(catalog_db, library_root, import_root):
    rows = query_rows(
        catalog_db,
        """
        SELECT
          p.id,
          p.library_id,
          p.folder_id,
          f.path AS import_folder_path,
          p.file_name,
          p.path,
          p.file_size,
          p.modified_at,
          p.imported_at,
          p.missing,
          p.unsupported,
          p.file_type,
          p.partial_hash,
          p.full_hash,
          pf.rating,
          pf.picked,
          pf.rejected,
          pf.color_label,
          pf.edited,
          pf.exported,
          pm.width,
          pm.height
        FROM photos p
        JOIN folders f ON f.id = p.folder_id
        LEFT JOIN photo_flags pf ON pf.photo_id = p.id
        LEFT JOIN photo_metadata pm ON pm.photo_id = p.id
        ORDER BY p.path
        """,
    )

    result = []
    for row in rows:
        source_path = Path(row["path"])
        source_stat = file_stat_record(source_path) if source_path.is_file() else {}
        result.append(
            {
                "id": row["id"],
                "library_id": row["library_id"],
                "folder_id": row["folder_id"],
                "import_folder_path": row["import_folder_path"],
                "file_name": row["file_name"],
                "path": row["path"],
                "file_size": row["file_size"],
                "modified_at": row["modified_at"],
                "imported_at": row["imported_at"],
                "missing": bool(row["missing"]),
                "unsupported": bool(row["unsupported"]),
                "file_type": row["file_type"],
                "partial_hash": row["partial_hash"],
                "full_hash": row["full_hash"],
                "flags": {
                    "exists": row["rating"] is not None,
                    "rating": row["rating"],
                    "picked": bool(row["picked"]) if row["picked"] is not None else None,
                    "rejected": bool(row["rejected"]) if row["rejected"] is not None else None,
                    "color_label": row["color_label"],
                    "edited": bool(row["edited"]) if row["edited"] is not None else None,
                    "exported": bool(row["exported"]) if row["exported"] is not None else None,
                },
                "metadata": {
                    "width": row["width"],
                    "height": row["height"],
                },
                "source_exists": source_path.is_file(),
                "references_import_root": path_is_relative_to(source_path, import_root),
                "outside_library_root": not path_is_relative_to(source_path, library_root),
                "current_source": source_stat,
                "file_size_matches_source": row["file_size"] == source_stat.get("size_bytes"),
                "modified_at_matches_source": row["modified_at"] == source_stat.get("modified_at"),
                "partial_hash_matches_source": row["partial_hash"] == source_stat.get("partial_hash"),
                "full_hash_matches_source": row["full_hash"] == source_stat.get("sha256"),
                "folder_path_matches_import_root": row["import_folder_path"] == import_root.as_posix(),
            }
        )
    return result


def import_action_logs(catalog_db):
    rows = query_rows(
        catalog_db,
        """
        SELECT action_type, actor_type, actor_id, subject_type, subject_id,
               side_effect_category, evidence_ref, payload_json, created_at
        FROM action_log
        WHERE action_type = 'import_reference'
        ORDER BY created_at, id
        """,
    )
    for row in rows:
        try:
            row["payload"] = json.loads(row.get("payload_json") or "{}")
        except json.JSONDecodeError:
            row["payload"] = None
    return rows


def absence_checks(rows, library_root, hidden_path, package_child_path):
    library_local_rows = [
        row["path"] for row in rows if path_is_relative_to(Path(row["path"]), library_root)
    ]
    hidden_or_package_rows = [
        row["path"]
        for row in rows
        if row["path"] in {hidden_path.as_posix(), package_child_path.as_posix()}
    ]
    return {
        "library_local_photo_paths": library_local_rows,
        "hidden_or_package_child_photo_paths": hidden_or_package_rows,
        "no_library_local_photo_paths": not library_local_rows,
        "hidden_and_package_child_skipped": not hidden_or_package_rows,
    }


def active_edit_states(catalog_db, photos_by_id):
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
        source = graph.get("source", {})
        fingerprint = source.get("fingerprint", {})
        photo = photos_by_id.get(row["photo_id"], {})
        result.append(
            {
                "id": row["id"],
                "photo_id": row["photo_id"],
                "active": bool(row["active"]),
                "source": source,
                "source_path_matches_photo": source.get("path") == photo.get("path"),
                "source_fingerprint_matches_photo": {
                    "file_size": fingerprint.get("file_size") == photo.get("file_size"),
                    "modified_at": fingerprint.get("modified_at") == photo.get("modified_at"),
                    "partial_hash": fingerprint.get("partial_hash") == photo.get("partial_hash"),
                    "full_hash": fingerprint.get("full_hash") == photo.get("full_hash"),
                },
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
                "source_full_hash": row["source_full_hash"],
                "settings": {
                    "format": settings.get("format"),
                    "color_profile": settings.get("color_profile"),
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


def command_tail(output, limit=80):
    lines = output.splitlines()
    return lines[-limit:]


def parse_args():
    parser = argparse.ArgumentParser(
        description="Record Q5.2 developer runtime evidence for library import by reference."
    )
    parser.add_argument(
        "--app",
        type=Path,
        default=DEFAULT_APP,
        help="Path to the built SilicaRAW.app artifact. Defaults to target/release/bundle/macos/SilicaRAW.app.",
    )
    parser.add_argument(
        "--scratch",
        type=Path,
        default=DEFAULT_SCRATCH,
        help="Scratch directory for fixtures and runtime output.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_SCRATCH / "library-import-reference-evidence.json",
        help="JSON evidence report output path.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    app = args.app if args.app.is_absolute() else ROOT / args.app
    scratch = args.scratch if args.scratch.is_absolute() else ROOT / args.scratch
    output = args.output if args.output.is_absolute() else ROOT / args.output
    fixtures = scratch / "fixtures"
    run_output = scratch / "run"
    library_root = run_output / "SilicaRAW Library"
    import_root = run_output / "Import Originals"
    export_root = run_output / "Exports"
    catalog_db = library_root / "catalog.db"

    failures = []
    if not app.is_dir():
        print(f"app artifact does not exist: {app}", file=sys.stderr)
        return 1

    if scratch.exists():
        shutil.rmtree(scratch)
    fixtures.mkdir(parents=True)
    run_output.mkdir(parents=True)

    generator = generate_fixtures(fixtures)
    if generator.returncode != 0:
        print("library import reference evidence failed: fixture generation failed", file=sys.stderr)
        print(generator.stdout, file=sys.stderr)
        print(generator.stderr, file=sys.stderr)
        return 1

    manifest = load_json(fixtures / "fixture-manifest.json")
    import_root.mkdir(parents=True, exist_ok=True)
    before_originals = seed_import_originals(fixtures, import_root)

    env = os.environ.copy()
    env["SILICARAW_RUNTIME_SMOKE_FIXTURES"] = str(fixtures)
    env["SILICARAW_RUNTIME_SMOKE_OUTPUT"] = str(run_output)
    command = [
        "cargo",
        "test",
        "-p",
        "silica-desktop",
        "tests::desktop_connected_runtime_smoke",
        "--",
        "--exact",
        "--nocapture",
    ]
    smoke = run(command, env=env)
    smoke_output = smoke.stdout + smoke.stderr
    marker_present = "phase-11 connected runtime smoke complete" in smoke_output
    if smoke.returncode != 0:
        failures.append("connected runtime smoke command failed")
    if "desktop_connected_runtime_smoke" not in smoke_output:
        failures.append("connected runtime smoke test name missing from output")
    if not marker_present:
        failures.append("connected runtime smoke completion marker missing")

    after_originals = finalize_original_hashes(before_originals)
    libraries = library_rows(catalog_db)
    folders = folder_rows(catalog_db)
    rows = catalog_rows(catalog_db, library_root, import_root)
    photos_by_id = {row["id"]: row for row in rows}
    actions = import_action_logs(catalog_db)
    absence = absence_checks(
        rows,
        library_root,
        import_root / "Recursive/.hidden.jpg",
        import_root / "Recursive/Archive.photoslibrary/package-child.jpg",
    )
    edits = active_edit_states(catalog_db, photos_by_id)
    exports = export_rows(catalog_db)
    row_names = {row["file_name"] for row in rows}
    required_rows = {
        "blocked-raw.DNG",
        "notes.txt",
        "recursive-child.jpg",
        "recursive-notes.txt",
        "reference-still-life.jpeg",
        "reference-urban.jpg",
    }
    import_payloads = [action.get("payload") for action in actions]
    expected_import_payloads = [
        {"recursive": False, "scanned_files": 4, "supported_files": 2, "unsupported_files": 2},
        {"recursive": True, "scanned_files": 6, "supported_files": 3, "unsupported_files": 3},
    ]
    checks = {
        "app_artifact_present": app.is_dir(),
        "fixture_manifest_present": (fixtures / "fixture-manifest.json").is_file(),
        "smoke_command_passed": smoke.returncode == 0,
        "smoke_completion_marker_present": marker_present,
        "catalog_db_exists": catalog_db.is_file(),
        "schema_version_12": schema_version(catalog_db) == 12,
        "library_row_points_to_library_root": any(
            row.get("id") == "local" and row.get("root_path") == library_root.as_posix()
            for row in libraries
        ),
        "import_folder_row_points_to_import_root": any(
            row.get("library_id") == "local"
            and row.get("path") == import_root.as_posix()
            and not row.get("missing")
            for row in folders
        ),
        "catalog_rows_present": len(rows) == len(required_rows),
        "required_import_rows_present": required_rows.issubset(row_names),
        "catalog_paths_reference_import_root": bool(rows)
        and all(row["references_import_root"] for row in rows),
        "catalog_paths_outside_library_root": bool(rows)
        and all(row["outside_library_root"] for row in rows),
        "catalog_sources_exist_after_workflow": bool(rows)
        and all(row["source_exists"] for row in rows),
        "catalog_fingerprints_match_sources": bool(rows)
        and all(
            row["file_size_matches_source"]
            and row["modified_at_matches_source"]
            and row["partial_hash_matches_source"]
            and row["full_hash_matches_source"]
            for row in rows
        ),
        "photo_flags_exist_for_all_rows": bool(rows) and all(row["flags"]["exists"] for row in rows),
        "hidden_and_package_child_not_cataloged": absence["hidden_and_package_child_skipped"],
        "no_library_local_photo_paths": absence["no_library_local_photo_paths"],
        "import_action_log_payloads_match": len(actions) == len(expected_import_payloads)
        and all(
            action.get("actor_type") == "core"
            and action.get("actor_id") == "local-alpha"
            and action.get("side_effect_category") == "catalog_reference"
            and action.get("evidence_ref") == import_root.as_posix()
            for action in actions
        )
        and import_payloads == expected_import_payloads,
        "edit_state_source_fingerprint_matches_photo": bool(edits)
        and all(
            edit["source_path_matches_photo"]
            and all(edit["source_fingerprint_matches_photo"].values())
            for edit in edits
        ),
        "export_settings_preserve_source_hash": bool(exports)
        and all(
            export["source_path_matches_photo"]
            and export["source_sha_matches_photo"]
            and export["source_sha_after_matches_before"]
            and export["source_original_hash_unchanged"]
            and export["output_exists"]
            for export in exports
        ),
        "original_hashes_unchanged": bool(after_originals)
        and all(record["ok"] for record in after_originals),
    }
    for check, passed in checks.items():
        if not passed:
            failures.append(check)

    report = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "smoke": "q5-library-import-reference",
        "source_commit": git_commit(),
        "app_artifact": artifact_record(app),
        "host": host_record(),
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
        "workflow_covered_by_runtime_smoke": [
            "create_library",
            "open_library",
            "import_folder_by_reference",
            "catalog_grid_query",
            "rating_pick_reject",
            "loupe_preview",
            "develop_exposure_contrast",
            "jpeg_srgb_export",
            "disposable_cache_clear",
            "reopen_library",
        ],
        "catalog": {
            "db_path": relative_path(catalog_db),
            "schema_version": schema_version(catalog_db),
            "libraries": libraries,
            "folders": folders,
            "row_count": len(rows),
            "rows": rows,
            "absence_checks": absence,
            "import_action_logs": actions,
            "active_edit_states": edits,
            "exports": exports,
        },
        "original_hashes": after_originals,
        "checks": checks,
        "rust_smoke": {
            "returncode": smoke.returncode,
            "completion_marker_present": marker_present,
            "stdout_stderr_tail": command_tail(smoke_output),
        },
        "known_limitations": KNOWN_LIMITATIONS,
        "failures": failures,
    }

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if failures:
        print(f"library import reference evidence failed; report written to {relative_path(output)}", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(f"library import reference evidence report written to {relative_path(output)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
