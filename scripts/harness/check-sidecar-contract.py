#!/usr/bin/env python3
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SIDECAR_SCHEMA = ROOT / "schemas/sidecar.schema.json"
SCHEMA_REFERENCE = ROOT / "docs/19_Schema_Reference.md"
STORAGE_SPEC = ROOT / "docs/10_Data_Model_and_Storage_Specification.md"
CATALOG_WIKI = ROOT / "docs/wiki/topics/catalog.md"
PHASE_10_DESIGN = ROOT / "docs/superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md"


def require(condition, message, failures):
    if not condition:
        failures.append(message)


def read_text(path, failures):
    try:
        return path.read_text(encoding="utf-8")
    except Exception as exc:
        failures.append(f"failed to read {path.relative_to(ROOT)}: {exc}")
        return ""


def load_json(path, failures):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        failures.append(f"failed to load {path.relative_to(ROOT)}: {exc}")
        return {}


def main():
    failures = []
    schema = load_json(SIDECAR_SCHEMA, failures)
    schema_reference = read_text(SCHEMA_REFERENCE, failures)
    storage_spec = read_text(STORAGE_SPEC, failures)
    catalog_wiki = read_text(CATALOG_WIKI, failures)
    phase_10_design = read_text(PHASE_10_DESIGN, failures)

    properties = schema.get("properties", {})
    flags = properties.get("flags", {})
    flag_properties = flags.get("properties", {})
    required_flags = flags.get("required", [])

    require(
        properties.get("schema", {}).get("const") == "silica.sidecar",
        "sidecar schema marker must stay silica.sidecar",
        failures,
    )
    require(
        properties.get("version", {}).get("const") == 1,
        "sidecar schema version must stay v1",
        failures,
    )
    require(
        set(required_flags) == {"rating", "picked", "rejected", "color_label"},
        "sidecar.flags required fields must be exactly rating/picked/rejected/color_label",
        failures,
    )
    require(
        set(flag_properties.keys()) == {"rating", "picked", "rejected", "color_label"},
        "sidecar.flags properties must be exactly rating/picked/rejected/color_label",
        failures,
    )
    require("edited" not in flag_properties, "sidecar.flags must not contain edited", failures)
    require("exported" not in flag_properties, "sidecar.flags must not contain exported", failures)
    require("exports" not in flag_properties, "sidecar.flags must not contain exports", failures)
    require(
        "sidecar.flags" in schema_reference and "intentionally limited" in schema_reference,
        "schema reference must preserve sidecar.flags scope",
        failures,
    )
    require(
        "Catalog rebuild rule" in storage_spec,
        "storage spec must preserve rebuild rule language",
        failures,
    )
    require(
        "photo_flags" in catalog_wiki and "live in-app authority" in catalog_wiki,
        "catalog wiki must preserve catalog authority language",
        failures,
    )
    require(
        "<library_root>/sidecars/<photo_id>.silicaraw.sidecar.json" in phase_10_design,
        "Phase 10 design must preserve library-local sidecar path",
        failures,
    )
    require(
        "Do not write sidecars next to original photo files" in phase_10_design,
        "Phase 10 design must block next-to-original sidecars",
        failures,
    )
    require(
        "Phase 10 sidecars are not evidence containers" in phase_10_design,
        "Phase 10 design must block proof payloads in sidecars",
        failures,
    )

    if failures:
        for failure in failures:
            print(f"sidecar contract check failed: {failure}", file=sys.stderr)
        return 1

    print("sidecar contract ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
