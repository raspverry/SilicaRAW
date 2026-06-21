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


def clamp_channel(value):
    return max(0, min(255, int(round(value))))


def blend_channel(base, overlay, amount):
    return base + (overlay - base) * amount


def blend_rgb(base, overlay, amount):
    amount = max(0.0, min(1.0, amount))
    return tuple(clamp_channel(blend_channel(base[index], overlay[index], amount)) for index in range(3))


def pseudo_noise(x, y, seed):
    value = (x * 73_856_093) ^ (y * 19_349_663) ^ (seed * 83_492_791)
    value ^= value >> 13
    value *= 1_274_126_177
    return (value & 255) / 255.0


def vignette(base, x, y, width, height):
    nx = (x / max(width - 1, 1) - 0.5) * 2.0
    ny = (y / max(height - 1, 1) - 0.5) * 2.0
    falloff = max(0.0, 1.0 - 0.28 * (nx * nx + ny * ny))
    return tuple(clamp_channel(channel * falloff) for channel in base)


def add_noise(base, x, y, seed, strength=8):
    noise = (pseudo_noise(x, y, seed) - 0.5) * strength
    return tuple(clamp_channel(channel + noise) for channel in base)


def reference_urban_pixel(x, y, width, height):
    nx = x / max(width - 1, 1)
    ny = y / max(height - 1, 1)

    if ny < 0.42:
        horizon = ny / 0.42
        base = blend_rgb((34, 53, 72), (164, 128, 101), horizon)
        cloud = max(0.0, 1.0 - abs(ny - 0.22 - 0.025 * pseudo_noise(x // 16, y // 9, 1)) * 18.0)
        base = blend_rgb(base, (188, 176, 158), cloud * 0.18)
    else:
        road_center = 0.5
        road_half_width = 0.08 + (ny - 0.42) * 0.72
        on_road = abs(nx - road_center) < road_half_width
        if on_road:
            distance = (ny - 0.42) / 0.58
            base = blend_rgb((49, 52, 54), (73, 72, 68), distance)
            lane_width = 0.005 + distance * 0.006
            lane = abs(nx - road_center) < lane_width and int((ny * height) // 36) % 2 == 0
            if lane:
                base = blend_rgb(base, (225, 204, 151), 0.7)
        else:
            left_side = nx < road_center
            side = (road_center - nx) if left_side else (nx - road_center)
            base = blend_rgb((58, 46, 42), (26, 31, 38), min(1.0, side * 2.1))

    buildings = [
        (0.02, 0.18, 0.20, 0.86, (54, 50, 48)),
        (0.20, 0.12, 0.36, 0.75, (66, 58, 50)),
        (0.64, 0.16, 0.80, 0.78, (47, 55, 59)),
        (0.80, 0.08, 0.97, 0.88, (39, 48, 60)),
    ]
    for x0, y0, x1, y1, color in buildings:
        if x0 <= nx <= x1 and y0 <= ny <= y1:
            wall_light = 0.85 + 0.18 * (1.0 - ny)
            base = tuple(clamp_channel(channel * wall_light) for channel in color)
            wx = int((nx - x0) / max(x1 - x0, 0.01) * 8)
            wy = int((ny - y0) / max(y1 - y0, 0.01) * 12)
            window_x = 0.24 < (((nx - x0) * 8) % 1.0) < 0.74
            window_y = 0.22 < (((ny - y0) * 12) % 1.0) < 0.62
            lit = ((wx * 7 + wy * 11 + int(x0 * 100)) % 5) in (0, 2)
            if window_x and window_y and lit:
                base = blend_rgb(base, (236, 180, 96), 0.72)

    for cx, cy, radius, color in [
        (0.30, 0.56, 0.070, (236, 172, 91)),
        (0.70, 0.50, 0.055, (122, 190, 211)),
        (0.49, 0.66, 0.038, (242, 215, 154)),
    ]:
        distance = ((nx - cx) ** 2 + ((ny - cy) * 1.25) ** 2) ** 0.5
        glow = max(0.0, 1.0 - distance / radius)
        base = blend_rgb(base, color, glow * 0.42)

    base = vignette(base, x, y, width, height)
    return add_noise(base, x, y, 17, 7)


def reference_still_life_pixel(x, y, width, height):
    nx = x / max(width - 1, 1)
    ny = y / max(height - 1, 1)
    base = blend_rgb((64, 69, 72), (154, 139, 118), ny * 0.85)

    if 0.08 < nx < 0.45 and 0.08 < ny < 0.54:
        window_light = 1.0 - abs(nx - 0.26) * 1.2 - abs(ny - 0.28) * 0.9
        base = blend_rgb(base, (196, 205, 195), max(0.0, window_light) * 0.55)

    if ny > 0.58:
        table_gradient = (ny - 0.58) / 0.42
        base = blend_rgb((83, 68, 57), (129, 102, 78), table_gradient)

    cup_dx = (nx - 0.54) / 0.145
    cup_dy = (ny - 0.55) / 0.18
    if cup_dx * cup_dx + cup_dy * cup_dy < 1.0:
        base = blend_rgb((196, 188, 176), (133, 124, 115), max(0.0, cup_dy + 0.2) * 0.45)
        rim = abs(cup_dx * cup_dx + ((ny - 0.43) / 0.045) ** 2 - 1.0)
        if rim < 0.17:
            base = blend_rgb(base, (234, 228, 216), 0.65)
    coffee_dx = (nx - 0.54) / 0.112
    coffee_dy = (ny - 0.43) / 0.037
    if coffee_dx * coffee_dx + coffee_dy * coffee_dy < 1.0:
        base = blend_rgb(base, (72, 49, 35), 0.88)

    if 0.18 < nx < 0.40 and 0.68 < ny < 0.84:
        base = blend_rgb(base, (181, 170, 148), 0.78)
        line = abs((ny - 0.76) - (nx - 0.29) * 0.28)
        if line < 0.008:
            base = blend_rgb(base, (78, 76, 73), 0.55)

    for cx, cy, sx, sy, color in [
        (0.72, 0.36, 0.030, 0.12, (67, 95, 77)),
        (0.78, 0.28, 0.035, 0.11, (74, 113, 88)),
        (0.68, 0.24, 0.027, 0.10, (89, 126, 96)),
    ]:
        leaf = ((nx - cx) / sx) ** 2 + ((ny - cy) / sy) ** 2
        if leaf < 1.0:
            base = blend_rgb(base, color, 0.75)

    base = vignette(base, x, y, width, height)
    return add_noise(base, x, y, 29, 6)


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

    urban_ppm = source / "reference-urban.ppm"
    urban_jpg = supported / "reference-urban.jpg"
    write_ppm(urban_ppm, 720, 480, reference_urban_pixel)
    convert_ppm_to_jpeg(urban_ppm, urban_jpg)

    still_life_ppm = source / "reference-still-life.ppm"
    still_life_jpeg = supported / "reference-still-life.jpeg"
    write_ppm(still_life_ppm, 640, 640, reference_still_life_pixel)
    convert_ppm_to_jpeg(still_life_ppm, still_life_jpeg)

    shutil.rmtree(source)
    for relative_path in [
        Path("supported/reference-urban.jpg"),
        Path("supported/reference-still-life.jpeg"),
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
                "import_supported": False,
                "preview_status": "unsupported",
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
