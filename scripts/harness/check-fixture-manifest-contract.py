#!/usr/bin/env python3
import json
import sys
from pathlib import PurePosixPath, Path


ROOT = Path(__file__).resolve().parents[2]
SCHEMA = ROOT / "schemas/fixture_manifest.schema.json"
EXAMPLE = ROOT / "schemas/fixture_manifest.example.json"
RAW_DOC = ROOT / "docs/wiki/topics/raw-decoding.md"
COLOR_DOC = ROOT / "docs/wiki/topics/color-management.md"
SCHEMA_REFERENCE = ROOT / "docs/19_Schema_Reference.md"

MANIFEST_KINDS = {"synthetic-local-alpha", "raw-fixtures", "color-fixtures", "mixed"}
RAW_CLASSES = {"A", "B", "C", "D", "E"}
CLASS_F_SUBCLASSES = {"srgb_jpeg", "display_p3_jpeg", "untagged_jpeg"}
LOWER_HEX = set("0123456789abcdef")


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def load_json(path, failures):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        failures.append(f"failed to load {path.relative_to(ROOT)}: {exc}")
        return {}


def is_safe_relative_path(value):
    if not isinstance(value, str) or not value:
        return False
    if "\\" in value or "//" in value:
        return False
    if any(part in {"", ".", ".."} for part in value.split("/")):
        return False
    path = PurePosixPath(value)
    return not path.is_absolute()


def is_lower_sha256(value):
    return isinstance(value, str) and len(value) == 64 and all(char in LOWER_HEX for char in value)


def validate_schema_contract(schema, failures):
    require(schema.get("schema") is None, "schema file must not be a manifest instance", failures)
    require(schema.get("properties", {}).get("schema", {}).get("const") == "silica.fixture_manifest", "schema must require fixture manifest marker", failures)
    require(schema.get("properties", {}).get("version", {}).get("const") == 1, "schema must require version 1", failures)

    manifest_kind_enum = set(schema.get("properties", {}).get("manifest_kind", {}).get("enum", []))
    require(MANIFEST_KINDS.issubset(manifest_kind_enum), "schema must preserve manifest_kind enum values", failures)

    defs = schema.get("$defs", {})
    for name in [
        "source_policy",
        "source",
        "license",
        "privacy",
        "integrity",
        "media",
        "expected_app_state",
        "expected_probe_state",
        "raw",
        "decode_gate",
        "color",
        "profile_expectation",
        "fixture",
    ]:
        require(name in defs, f"schema must define $defs/{name}", failures)

    fixture = defs.get("fixture", {})
    fixture_required = set(fixture.get("required", []))
    for key in [
        "id",
        "class",
        "kind",
        "relative_path",
        "availability",
        "source",
        "license",
        "privacy",
        "integrity",
        "media",
        "expected_app_state",
        "expected_probe_state",
    ]:
        require(key in fixture_required, f"fixture schema must require {key}", failures)

    fixture_properties = fixture.get("properties", {})
    require({"A", "B", "C", "D", "E", "F"}.issubset(set(fixture_properties.get("class", {}).get("enum", []))), "fixture class enum must include A-F", failures)
    require(
        {"raw", "tagged_raster", "untagged_raster", "unsupported", "raw_blocked_placeholder"}.issubset(
            set(fixture_properties.get("kind", {}).get("enum", []))
        ),
        "fixture kind enum must include RAW/color placeholder values",
        failures,
    )
    require(
        {"generated", "committed", "local_ignored", "external_reference_only"}.issubset(
            set(fixture_properties.get("availability", {}).get("enum", []))
        ),
        "fixture availability enum must include provenance states",
        failures,
    )

    app_state = defs.get("expected_app_state", {}).get("properties", {})
    require(
        {"ready_by_reference", "raw_decode_blocked", "unsupported", "missing", "not_probed"}.issubset(
            set(app_state.get("preview_status", {}).get("enum", []))
        ),
        "preview_status enum must preserve fixture states",
        failures,
    )

    probe_state = defs.get("expected_probe_state", {}).get("properties", {})
    require(
        {"unverified", "blocked_pending_task_12", "blocked_pending_task_13"}.issubset(
            set(probe_state.get("state", {}).get("enum", []))
        ),
        "probe_state enum must preserve blocked states",
        failures,
    )

    raw_class_rule = {}
    for rule in fixture.get("allOf", []):
        class_enum = set(rule.get("if", {}).get("properties", {}).get("class", {}).get("enum", []))
        if RAW_CLASSES.issubset(class_enum):
            raw_class_rule = rule
            break
    raw_class_rule_text = json.dumps(raw_class_rule, sort_keys=True)
    require(raw_class_rule, "schema must define a RAW class A-E conditional rule", failures)
    require("blocked_pending_task_12" in raw_class_rule_text, "schema must keep RAW classes blocked until Task 12", failures)
    require("raw_decode_blocked" in raw_class_rule_text, "schema must keep RAW class preview_status blocked", failures)
    require("raw_blocked_placeholder" not in raw_class_rule_text, "schema must not allow raw_blocked_placeholder as a RAW class fixture kind", failures)


def validate_manifest(manifest, failures):
    require(manifest.get("schema") == "silica.fixture_manifest", "manifest schema must be silica.fixture_manifest", failures)
    require(manifest.get("version") == 1, "manifest version must be 1", failures)
    require(manifest.get("manifest_kind") in MANIFEST_KINDS, "manifest_kind must be known", failures)
    require(isinstance(manifest.get("fixtures"), list) and manifest["fixtures"], "fixtures must be a non-empty list", failures)
    require(isinstance(manifest.get("expected_source_hashes"), dict), "expected_source_hashes must be an object", failures)

    fixtures = manifest.get("fixtures") if isinstance(manifest.get("fixtures"), list) else []
    expected_source_hashes = manifest.get("expected_source_hashes") if isinstance(manifest.get("expected_source_hashes"), dict) else {}
    required = [
        "id",
        "class",
        "kind",
        "relative_path",
        "availability",
        "source",
        "license",
        "privacy",
        "integrity",
        "media",
        "expected_app_state",
        "expected_probe_state",
    ]
    seen_ids = set()
    seen_raw_classes = set()
    seen_class_f_subclasses = set()

    for index, fixture in enumerate(fixtures):
        if not isinstance(fixture, dict):
            failures.append(f"fixture {index} must be an object")
            continue

        fixture_id = fixture.get("id") if fixture.get("id") else f"fixture {index}"
        for key in required:
            require(key in fixture, f"{fixture_id} missing {key}", failures)

        if fixture.get("id") in seen_ids:
            failures.append(f"{fixture_id} id must be unique")
        seen_ids.add(fixture.get("id"))

        relative_path = fixture.get("relative_path")
        require(
            is_safe_relative_path(relative_path),
            f"{fixture_id} relative_path must be a non-empty relative POSIX path without absolute prefix, . or .. path components, backslashes, or repeated /",
            failures,
        )

        integrity = fixture.get("integrity") if isinstance(fixture.get("integrity"), dict) else {}
        sha256 = integrity.get("sha256")
        require(is_lower_sha256(sha256), f"{fixture_id} integrity.sha256 must be 64 lowercase hex characters", failures)
        require(expected_source_hashes.get(relative_path) == sha256, f"{fixture_id} expected_source_hashes entry must match integrity.sha256", failures)

        license_info = fixture.get("license") if isinstance(fixture.get("license"), dict) else {}
        privacy = fixture.get("privacy") if isinstance(fixture.get("privacy"), dict) else {}
        if fixture.get("availability") == "committed":
            require(license_info.get("name") != "Unknown", f"{fixture_id} committed fixture license cannot be Unknown", failures)
            require(privacy.get("is_user_photo") is not True, f"{fixture_id} committed fixture cannot be a user photo", failures)

        fixture_class = fixture.get("class")
        kind = fixture.get("kind")
        raw = fixture.get("raw") if isinstance(fixture.get("raw"), dict) else {}

        if kind == "raw":
            require("raw" in fixture, f"{fixture_id} raw fixture must include raw", failures)
            require("decode_gate" in fixture, f"{fixture_id} raw fixture must include decode_gate", failures)

        if fixture_class in RAW_CLASSES:
            seen_raw_classes.add(fixture_class)
            decode_gate = fixture.get("decode_gate") if isinstance(fixture.get("decode_gate"), dict) else {}
            expected_probe_state = fixture.get("expected_probe_state") if isinstance(fixture.get("expected_probe_state"), dict) else {}
            require("raw" in fixture, f"{fixture_id} RAW class fixture must include raw", failures)
            require("decode_gate" in fixture, f"{fixture_id} RAW class fixture must include decode_gate", failures)
            require(decode_gate.get("state") == "blocked_pending_task_12", f"{fixture_id} RAW decode gate must stay blocked_pending_task_12", failures)
            require(
                expected_probe_state.get("actual_result") in (None, "not_recorded"),
                f"{fixture_id} must not record actual RAW probe results in Task 10.1",
                failures,
            )
            require(kind != "raw_blocked_placeholder", f"{fixture_id} raw_blocked_placeholder must not be used as a real RAW fixture", failures)

        if fixture_class == "C":
            require(raw.get("format") == "raf", f"{fixture_id} Class C raw.format must be raf", failures)
        if fixture_class == "D":
            require(raw.get("format") == "dng", f"{fixture_id} Class D raw.format must be dng", failures)
            require(raw.get("apple_proraw") is True, f"{fixture_id} Class D raw.apple_proraw must be true", failures)

        if fixture_class == "F":
            color = fixture.get("color") if isinstance(fixture.get("color"), dict) else {}
            profile = fixture.get("profile_expectation") if isinstance(fixture.get("profile_expectation"), dict) else {}
            require("color" in fixture, f"{fixture_id} Color Class F fixture must include color", failures)
            require("profile_expectation" in fixture, f"{fixture_id} Color Class F fixture must include profile_expectation", failures)
            subclass = color.get("subclass")
            seen_class_f_subclasses.add(subclass)
            if subclass in {"srgb_jpeg", "display_p3_jpeg"}:
                require(profile.get("embedded_icc") is True, f"{fixture_id} tagged Class F fixture must embed ICC", failures)
            if subclass == "untagged_jpeg":
                require(profile.get("embedded_icc") is False, f"{fixture_id} untagged Class F fixture must not embed ICC", failures)
                require(profile.get("untagged_policy") == "assume_srgb", f"{fixture_id} untagged Class F policy must assume_srgb", failures)
            require(profile.get("color_correctness_proven") is not True, f"{fixture_id} Class F must not claim color correctness proof", failures)

    require(RAW_CLASSES.issubset(seen_raw_classes), "example must include RAW classes A, B, C, D, and E", failures)
    require(
        CLASS_F_SUBCLASSES.issubset(seen_class_f_subclasses),
        "Class F example must include srgb_jpeg, display_p3_jpeg, and untagged_jpeg",
        failures,
    )


def read_text(path, failures):
    try:
        return path.read_text(encoding="utf-8")
    except Exception as exc:
        failures.append(f"failed to read {path.relative_to(ROOT)}: {exc}")
        return ""


def main():
    failures = []
    schema = load_json(SCHEMA, failures)
    manifest = load_json(EXAMPLE, failures)

    validate_schema_contract(schema, failures)
    validate_manifest(manifest, failures)

    raw_doc = read_text(RAW_DOC, failures)
    color_doc = read_text(COLOR_DOC, failures)
    schema_reference = read_text(SCHEMA_REFERENCE, failures)
    require("fixture_manifest.schema.json" in schema_reference, "schema reference must list fixture manifest schema", failures)
    require("no committed legal RAW fixture corpus" in raw_doc, "RAW docs must state no committed legal RAW fixture corpus", failures)
    require("RAW support claims remain blocked" in raw_doc, "RAW docs must preserve support-claim boundary", failures)
    require("Color Class F" in color_doc, "color docs must describe Color Class F", failures)
    require("do not prove color correctness" in color_doc, "color docs must preserve color-correctness boundary", failures)

    if failures:
        for failure in failures:
            print(f"fixture manifest contract failed: {failure}", file=sys.stderr)
        return 1

    print("fixture manifest contract ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
