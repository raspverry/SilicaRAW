#!/usr/bin/env python3
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_ppm(path, width, height, pixel_fn):
    lines = [
        "P3",
        "# SilicaRAW synthetic legal fixture source",
        f"{width} {height}",
        "255",
    ]
    for y in range(height):
        row = []
        for x in range(width):
            row.extend(str(channel) for channel in pixel_fn(x, y, width, height))
        lines.append(" ".join(row))
    path.write_text("\n".join(lines) + "\n", encoding="ascii")


def convert_ppm_to_jpeg(ppm_path, jpeg_path):
    sips = shutil.which("sips")
    if not sips:
        raise RuntimeError("macOS sips command is required to generate JPEG fixtures")
    result = subprocess.run(
        [sips, "-s", "format", "jpeg", str(ppm_path), "--out", str(jpeg_path)],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"sips failed while creating {jpeg_path.name}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )


def record_fixture(fixtures, root, relative_path, role, media_type, expected):
    path = root / relative_path
    fixtures.append(
        {
            "relative_path": relative_path.as_posix(),
            "role": role,
            "media_type": media_type,
            "extension": relative_path.suffix.lower(),
            "size_bytes": path.stat().st_size,
            "sha256": sha256_file(path),
            "expected": expected,
        }
    )


def create_supported_jpegs(output, fixtures):
    supported = output / "supported"
    source = output / "_sources"
    supported.mkdir(parents=True, exist_ok=True)
    source.mkdir(parents=True, exist_ok=True)

    gradient_ppm = source / "synthetic-gradient.ppm"
    gradient_jpg = supported / "synthetic-gradient.jpg"
    write_ppm(
        gradient_ppm,
        96,
        64,
        lambda x, y, width, height: (
            int(32 + (x / max(width - 1, 1)) * 192),
            int(48 + (y / max(height - 1, 1)) * 160),
            int(224 - (x / max(width - 1, 1)) * 96),
        ),
    )
    convert_ppm_to_jpeg(gradient_ppm, gradient_jpg)

    checker_ppm = source / "synthetic-checker.ppm"
    checker_jpeg = supported / "synthetic-checker.jpeg"
    write_ppm(
        checker_ppm,
        80,
        80,
        lambda x, y, _width, _height: (
            (240, 235, 222)
            if ((x // 10) + (y // 10)) % 2 == 0
            else (48, 68, 78)
        ),
    )
    convert_ppm_to_jpeg(checker_ppm, checker_jpeg)

    shutil.rmtree(source)
    for relative_path in [
        Path("supported/synthetic-gradient.jpg"),
        Path("supported/synthetic-checker.jpeg"),
    ]:
        record_fixture(
            fixtures,
            output,
            relative_path,
            "supported-jpeg",
            "image/jpeg",
            {
                "import_supported": True,
                "preview_status": "ready_by_reference",
                "visible_alpha_path": True,
            },
        )


def create_unsupported_files(output, fixtures):
    unsupported = output / "unsupported"
    unsupported.mkdir(parents=True, exist_ok=True)

    notes = unsupported / "notes.txt"
    notes.write_text(
        "SilicaRAW unsupported text fixture.\n"
        "This file is intentionally not a photo and should import as unsupported.\n",
        encoding="utf-8",
    )
    webp = unsupported / "unsupported-placeholder.webp"
    webp.write_bytes(
        b"RIFF\x1c\x00\x00\x00WEBPVP8 "
        b"SilicaRAW unsupported WebP placeholder; not a user photo.\n"
    )

    for relative_path, media_type in [
        (Path("unsupported/notes.txt"), "text/plain"),
        (Path("unsupported/unsupported-placeholder.webp"), "image/webp-placeholder"),
    ]:
        record_fixture(
            fixtures,
            output,
            relative_path,
            "unsupported",
            media_type,
            {
                "import_supported": False,
                "preview_status": "unsupported",
                "visible_alpha_path": False,
            },
        )


def create_raw_placeholders(output, fixtures):
    raw_dir = output / "raw-blocked"
    raw_dir.mkdir(parents=True, exist_ok=True)

    for filename in ["blocked-raw.DNG", "blocked-raw.RAF"]:
        path = raw_dir / filename
        path.write_text(
            "SilicaRAW RAW placeholder.\n"
            "This is a legal text placeholder used to exercise RAW-blocked UI states.\n"
            "It is not camera data and must never be treated as a decodable RAW sample.\n",
            encoding="utf-8",
        )
        record_fixture(
            fixtures,
            output,
            Path("raw-blocked") / filename,
            "raw-blocked-placeholder",
            "text/plain; raw-placeholder",
            {
                "import_supported": True,
                "preview_status": "raw_decode_blocked",
                "visible_alpha_path": False,
            },
        )


def write_manifest(output, fixtures, include_raw_placeholders):
    fixtures.sort(key=lambda fixture: fixture["relative_path"])
    manifest = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "generator": "scripts/harness/generate-legal-fixtures.py",
        "license": "SilicaRAW synthetic fixture data generated by project scripts; no user photos.",
        "source_policy": (
            "Fixtures are generated from deterministic synthetic pixel patterns or explicit placeholder text. "
            "The repository does not commit user photos for this QA set."
        ),
        "include_raw_placeholders": include_raw_placeholders,
        "fixtures": fixtures,
        "expected_source_hashes": {
            fixture["relative_path"]: fixture["sha256"] for fixture in fixtures
        },
    }
    manifest_path = output / "fixture-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest_path


def parse_args():
    parser = argparse.ArgumentParser(description="Generate legal local-alpha QA fixtures.")
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / ".tmp/legal-qa-fixtures",
        help="Fixture output directory. Defaults to .tmp/legal-qa-fixtures.",
    )
    parser.add_argument(
        "--include-raw-placeholders",
        action="store_true",
        help="Also generate explicit non-camera RAW-blocked placeholder files.",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    output = args.output if args.output.is_absolute() else ROOT / args.output
    output.mkdir(parents=True, exist_ok=True)

    fixtures = []
    try:
        create_supported_jpegs(output, fixtures)
        create_unsupported_files(output, fixtures)
        if args.include_raw_placeholders:
            create_raw_placeholders(output, fixtures)
        manifest_path = write_manifest(output, fixtures, args.include_raw_placeholders)
    except Exception as exc:
        print(f"failed to generate legal QA fixtures: {exc}", file=sys.stderr)
        return 1

    print(f"generated {len(fixtures)} fixtures")
    print(manifest_path.relative_to(ROOT) if manifest_path.is_relative_to(ROOT) else manifest_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
