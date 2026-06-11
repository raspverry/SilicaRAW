# Sidecar v1 Read/Write Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Task 10.3 by writing and reading validated library-local sidecars for edit graph state and portable culling flags without writing next to originals, adding automatic sync, or adding RAW/color/export proof.

**Architecture:** `silica-storage` owns sidecar paths, catalog reads, JSON construction, schema-aware validation, atomic file writes, validated reads, and `sidecar_status` updates. `silica-core` exposes only thin workflow wrappers over storage. Static trust guardrails stay in `scripts/harness/`, and docs record the sidecar boundary for future rebuild, backup, and public trust work.

**Tech Stack:** Rust 2021, existing `rusqlite`, existing `serde_json`, existing `silica-edit` validation helpers, Python 3 standard library harness checks, Markdown docs. Do not add external dependencies for Task 10.3.

---

## File Structure

- Modify: `crates/silica-storage/src/lib.rs`
  - Add sidecar path helper, payload construction, validation helpers, atomic write, validated read, and storage tests.
- Modify: `crates/silica-core/src/lib.rs`
  - Add command-facing sidecar write/read wrappers and core tests.
- Create: `scripts/harness/check-sidecar-contract.py`
  - Static guard for sidecar schema/docs contract.
- Modify: `scripts/harness/check.sh`
  - Run the new sidecar contract guard after the golden tolerance guard.
- Modify: `crates/silica-storage/README.md`
  - Record that Task 10.3 owns library-local sidecars and does not write next to originals.
- Modify: `crates/silica-core/README.md`
  - Record that Core exposes thin sidecar workflow wrappers only.
- Modify: `docs/wiki/topics/catalog.md`
  - Move sidecar read/write from not-implemented to implemented foundation after the code exists.
- Modify: `docs/wiki/topics/data-safety.md`
  - Record original-file safety and sidecar recovery boundaries.
- Modify: `docs/wiki/roadmaps/post-alpha-product-roadmap.md`
  - Mark Task 10.3 completed after implementation.
- Modify: `docs/wiki/log.md`
  - Add an append-only Task 10.3 entry.

Do not modify `schemas/sidecar.schema.json` in Task 10.3 unless implementation discovers a direct contradiction with `docs/19_Schema_Reference.md`. If the schema must change, stop and create a schema decision PR first.

## Task 1: Add Static Sidecar Contract Guard

**Files:**
- Create: `scripts/harness/check-sidecar-contract.py`
- Modify: `scripts/harness/check.sh`

- [ ] **Step 1: Add the failing harness expectation**

Create `scripts/harness/check-sidecar-contract.py` with this exact structure:

```python
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

    require(properties.get("schema", {}).get("const") == "silica.sidecar", "sidecar schema marker must stay silica.sidecar", failures)
    require(properties.get("version", {}).get("const") == 1, "sidecar schema version must stay v1", failures)
    require(set(required_flags) == {"rating", "picked", "rejected", "color_label"}, "sidecar.flags required fields must be exactly rating/picked/rejected/color_label", failures)
    require(set(flag_properties.keys()) == {"rating", "picked", "rejected", "color_label"}, "sidecar.flags properties must be exactly rating/picked/rejected/color_label", failures)
    require("edited" not in flag_properties, "sidecar.flags must not contain edited", failures)
    require("exported" not in flag_properties, "sidecar.flags must not contain exported", failures)
    require("exports" not in flag_properties, "sidecar.flags must not contain exports", failures)
    require("sidecar.flags is intentionally limited" in schema_reference, "schema reference must preserve sidecar.flags scope", failures)
    require("Catalog rebuild rule" in storage_spec, "storage spec must preserve rebuild rule language", failures)
    require("photo_flags is the live in-app authority" in catalog_wiki, "catalog wiki must preserve catalog authority language", failures)
    require("<library_root>/sidecars/<photo_id>.silicaraw.sidecar.json" in phase_10_design, "Phase 10 design must preserve library-local sidecar path", failures)
    require("Do not write sidecars next to original photo files" in phase_10_design, "Phase 10 design must block next-to-original sidecars", failures)
    require("Phase 10 sidecars are not evidence containers" in phase_10_design, "Phase 10 design must block proof payloads in sidecars", failures)

    if failures:
        for failure in failures:
            print(f"sidecar contract check failed: {failure}", file=sys.stderr)
        return 1

    print("sidecar contract ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 2: Run the new checker before wiring it into the harness**

Run:

```bash
python3 scripts/harness/check-sidecar-contract.py
```

Expected: `sidecar contract ok`.

- [ ] **Step 3: Wire the checker into `scripts/harness/check.sh`**

Insert this block after the golden tolerance policy check:

```bash
echo "==> Checking sidecar contract"
python3 scripts/harness/check-sidecar-contract.py
```

- [ ] **Step 4: Verify harness wiring**

Run:

```bash
scripts/harness/check.sh
```

Expected: command exits with status 0 and prints `sidecar contract ok`.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
git add scripts/harness/check-sidecar-contract.py scripts/harness/check.sh
git commit -m "test(sidecars): add sidecar contract guard"
```

Expected: commit contains only the new harness script and harness wiring.

## Task 2: Add Sidecar Path Validation

**Files:**
- Modify: `crates/silica-storage/src/lib.rs`

- [ ] **Step 1: Add failing storage tests**

Inside `crates/silica-storage/src/lib.rs`, add these tests near the other storage tests:

```rust
#[test]
fn resolves_sidecar_paths_under_library_sidecars_only() {
    let root = unique_library_root("sidecar-path");
    let path = sidecar_path_for_photo(&root, "photo_ABC-123.ok")
        .expect("valid sidecar path");

    assert_eq!(
        path,
        root.join("sidecars")
            .join("photo_ABC-123.ok.silicaraw.sidecar.json")
    );
    assert!(path.starts_with(root.join("sidecars")));
}

#[test]
fn rejects_unsafe_sidecar_photo_ids() {
    let root = unique_library_root("sidecar-invalid-id");
    for invalid in ["", "../photo", "folder/photo", "folder\\photo", "/tmp/photo", "photo\nid"] {
        assert!(
            sidecar_path_for_photo(&root, invalid).is_err(),
            "invalid photo id should be rejected: {invalid:?}"
        );
    }
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test -p silica-storage sidecar_path
```

Expected: fails because `sidecar_path_for_photo` does not exist.

- [ ] **Step 3: Add storage constants and error variants**

Near the existing library constants, add:

```rust
/// Library-local directory for portable sidecar JSON files.
pub const SIDECAR_DIRECTORY: &str = "sidecars";

/// Stable sidecar schema marker required by `schemas/sidecar.schema.json`.
pub const SIDECAR_SCHEMA: &str = "silica.sidecar";

/// Stable sidecar schema version for v0.1.
pub const SIDECAR_VERSION: i64 = 1;
```

Extend `LibraryStorageError` with:

```rust
    MissingPhoto(String),
    InvalidSidecarPhotoId(String),
    SidecarValidation(String),
```

Extend its `Display` match with:

```rust
            Self::MissingPhoto(photo_id) => write!(formatter, "missing catalog photo: {photo_id}"),
            Self::InvalidSidecarPhotoId(photo_id) => {
                write!(formatter, "invalid sidecar photo id: {photo_id:?}")
            }
            Self::SidecarValidation(message) => write!(formatter, "sidecar validation error: {message}"),
```

Extend `source()` by adding those variants to the `None` branch.

- [ ] **Step 4: Add path helper implementation**

Add this public helper below `open_local_library`:

```rust
/// Resolve the library-local sidecar path for a catalog photo id.
pub fn sidecar_path_for_photo(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<PathBuf, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    Ok(library_root_path
        .as_ref()
        .join(SIDECAR_DIRECTORY)
        .join(format!("{photo_id}.silicaraw.sidecar.json")))
}
```

Add this private helper near the other private helpers:

```rust
fn validate_sidecar_photo_id(photo_id: &str) -> Result<(), LibraryStorageError> {
    if photo_id.is_empty()
        || photo_id == "."
        || photo_id == ".."
        || photo_id.contains('/')
        || photo_id.contains('\\')
        || photo_id.contains("..")
        || photo_id.chars().any(|character| {
            !(character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.')
        })
    {
        return Err(LibraryStorageError::InvalidSidecarPhotoId(
            photo_id.to_string(),
        ));
    }

    Ok(())
}
```

- [ ] **Step 5: Run the focused test**

Run:

```bash
cargo test -p silica-storage sidecar_path
```

Expected: the two sidecar path tests pass.

- [ ] **Step 6: Commit Task 2**

Run:

```bash
git add crates/silica-storage/src/lib.rs
git commit -m "feat(storage): add sidecar path validation"
```

Expected: commit contains only storage path validation and its tests.

## Task 3: Add Sidecar Payload Construction and Validation

**Files:**
- Modify: `crates/silica-storage/src/lib.rs`

- [ ] **Step 1: Add failing payload tests**

Add this test after the sidecar path tests:

```rust
#[test]
fn builds_valid_sidecar_payload_with_flags_and_metadata_mirror() {
    let workspace = unique_library_root("sidecar-payload");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("purple".to_string()),
    )
    .expect("set flags");

    let sidecar = build_photo_sidecar_value(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("build sidecar");

    validate_sidecar_json(&sidecar).expect("validate sidecar");
    assert_eq!(sidecar["schema"], SIDECAR_SCHEMA);
    assert_eq!(sidecar["version"], SIDECAR_VERSION);
    assert_eq!(sidecar["photo"]["photo_id"], photo_id);
    assert_eq!(sidecar["flags"]["rating"], 5);
    assert_eq!(sidecar["flags"]["picked"], true);
    assert_eq!(sidecar["flags"]["rejected"], false);
    assert_eq!(sidecar["flags"]["color_label"], "purple");
    assert_eq!(sidecar["edit_graph"]["metadata"]["rating"], 5);
    assert_eq!(sidecar["edit_graph"]["metadata"]["picked"], true);
    assert_eq!(sidecar["edit_graph"]["metadata"]["rejected"], false);
    assert_eq!(sidecar["edit_graph"]["metadata"]["color_label"], "purple");
    assert!(sidecar["sync"]["sidecar_hash"].is_null());
    assert!(sidecar["flags"].get("edited").is_none());
    assert!(sidecar["flags"].get("exported").is_none());

    let connection = open_catalog(library.catalog_path).expect("open catalog");
    assert_eq!(
        count_edit_states(&connection),
        0,
        "building a sidecar for an unedited photo must not write edit_states"
    );

    remove_library_root(&workspace);
}

#[test]
fn rejects_sidecar_payload_for_invalid_color_label() {
    let workspace = unique_library_root("sidecar-invalid-label");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    connection
        .execute(
            "UPDATE photo_flags SET color_label = 'cyan' WHERE photo_id = ?1",
            params![photo_id],
        )
        .expect("force invalid catalog label");
    drop(connection);

    let error = build_photo_sidecar_value(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect_err("invalid sidecar label should fail");
    assert!(error.to_string().contains("unsupported sidecar color label"));

    remove_library_root(&workspace);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test -p silica-storage sidecar_payload
```

Expected: fails because payload helpers do not exist.

- [ ] **Step 3: Add private sidecar row and helper functions**

Add this private row type near `CacheRecord` and `ExportRecord`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct SidecarPhotoRow {
    photo_id: String,
    original_path: String,
    file_name: String,
    file_size: i64,
    modified_at: Option<String>,
    partial_hash: Option<String>,
    full_hash: Option<String>,
}
```

Add these helper functions near `load_active_edit_graph_or_default`:

```rust
fn load_sidecar_photo_row(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<SidecarPhotoRow>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id, path, file_name, file_size, modified_at, partial_hash, full_hash
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(SidecarPhotoRow {
                    photo_id: row.get(0)?,
                    original_path: row.get(1)?,
                    file_name: row.get(2)?,
                    file_size: row.get(3)?,
                    modified_at: row.get(4)?,
                    partial_hash: row.get(5)?,
                    full_hash: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

fn active_edit_state_id(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<String>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id
            FROM edit_states
            WHERE photo_id = ?1 AND active = 1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![photo_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(LibraryStorageError::from)
}
```

- [ ] **Step 4: Add color label conversion helpers**

Add:

```rust
fn edit_color_label_from_catalog(
    label: Option<&str>,
) -> Result<Option<silica_edit::ColorLabel>, LibraryStorageError> {
    match label {
        None => Ok(None),
        Some("red") => Ok(Some(silica_edit::ColorLabel::Red)),
        Some("orange") => Ok(Some(silica_edit::ColorLabel::Orange)),
        Some("yellow") => Ok(Some(silica_edit::ColorLabel::Yellow)),
        Some("green") => Ok(Some(silica_edit::ColorLabel::Green)),
        Some("blue") => Ok(Some(silica_edit::ColorLabel::Blue)),
        Some("purple") => Ok(Some(silica_edit::ColorLabel::Purple)),
        Some(other) => Err(LibraryStorageError::SidecarValidation(format!(
            "unsupported sidecar color label: {other}"
        ))),
    }
}

fn color_label_value(label: Option<&str>) -> serde_json::Value {
    match label {
        Some(label) => serde_json::Value::String(label.to_string()),
        None => serde_json::Value::Null,
    }
}
```

- [ ] **Step 5: Add sidecar payload construction**

Add this private helper:

```rust
fn build_photo_sidecar_value(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<serde_json::Value, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let photo = load_sidecar_photo_row(&connection, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;
    let flags = get_photo_flags(&library.root_path, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;
    let mut graph = load_active_edit_graph_or_default(&library.root_path, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;

    graph.app_version = Some(app_version.to_string());
    graph.metadata.rating = i64::from(flags.rating);
    graph.metadata.picked = flags.picked;
    graph.metadata.rejected = flags.rejected;
    graph.metadata.color_label = edit_color_label_from_catalog(flags.color_label.as_deref())?;
    silica_edit::validate_edit_graph(&graph)?;
    let edit_graph_json = serde_json::to_value(&graph)?;
    let written_at = current_timestamp_string();
    let catalog_edit_state_id = active_edit_state_id(&connection, photo_id)?;

    let value = serde_json::json!({
        "schema": SIDECAR_SCHEMA,
        "version": SIDECAR_VERSION,
        "app_version": app_version,
        "photo": {
            "photo_id": photo.photo_id,
            "original_path": photo.original_path,
            "file_name": photo.file_name,
            "fingerprint": {
                "file_size": photo.file_size,
                "modified_at": photo.modified_at.unwrap_or_else(|| "unknown".to_string()),
                "partial_hash": photo.partial_hash.unwrap_or_default(),
                "full_hash": photo.full_hash
            }
        },
        "edit_graph": edit_graph_json,
        "flags": {
            "rating": flags.rating,
            "picked": flags.picked,
            "rejected": flags.rejected,
            "color_label": color_label_value(flags.color_label.as_deref())
        },
        "sync": {
            "status": "in_sync",
            "catalog_edit_state_id": catalog_edit_state_id,
            "sidecar_hash": serde_json::Value::Null
        },
        "written_at": written_at
    });

    validate_sidecar_json(&value)?;
    Ok(value)
}
```

- [ ] **Step 6: Add sidecar validation helper**

Add this private helper:

```rust
fn validate_sidecar_json(value: &serde_json::Value) -> Result<(), LibraryStorageError> {
    let object = value
        .as_object()
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar root must be an object".to_string()))?;
    if object.get("schema").and_then(serde_json::Value::as_str) != Some(SIDECAR_SCHEMA) {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar schema marker must be silica.sidecar".to_string(),
        ));
    }
    if object.get("version").and_then(serde_json::Value::as_i64) != Some(SIDECAR_VERSION) {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar version must be 1".to_string(),
        ));
    }
    for required in ["app_version", "photo", "edit_graph", "flags", "sync", "written_at"] {
        if !object.contains_key(required) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar missing required field: {required}"
            )));
        }
    }

    let flags = object
        .get("flags")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.flags must be an object".to_string()))?;
    let allowed_flags = ["rating", "picked", "rejected", "color_label"];
    for key in flags.keys() {
        if !allowed_flags.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.flags contains unsupported field: {key}"
            )));
        }
    }
    if flags.get("rating").and_then(serde_json::Value::as_i64).map_or(true, |rating| !(0..=5).contains(&rating)) {
        return Err(LibraryStorageError::SidecarValidation("sidecar.flags.rating must be 0..=5".to_string()));
    }
    for key in ["picked", "rejected"] {
        if !flags.get(key).is_some_and(serde_json::Value::is_boolean) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.flags.{key} must be boolean"
            )));
        }
    }
    match flags.get("color_label") {
        Some(value) if value.is_null() => {}
        Some(value) => {
            let label = value.as_str().ok_or_else(|| {
                LibraryStorageError::SidecarValidation("sidecar.flags.color_label must be string or null".to_string())
            })?;
            edit_color_label_from_catalog(Some(label))?;
        }
        None => {
            return Err(LibraryStorageError::SidecarValidation(
                "sidecar.flags.color_label is required".to_string(),
            ));
        }
    }

    let edit_graph = object
        .get("edit_graph")
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.edit_graph is required".to_string()))?;
    silica_edit::validate_edit_graph_json(edit_graph)?;

    Ok(())
}
```

- [ ] **Step 7: Run focused payload tests**

Run:

```bash
cargo test -p silica-storage sidecar_payload
```

Expected: both payload tests pass.

- [ ] **Step 8: Commit Task 3**

Run:

```bash
git add crates/silica-storage/src/lib.rs
git commit -m "feat(storage): build validated sidecar payloads"
```

Expected: commit contains sidecar payload construction, validation helpers, and focused tests.

## Task 4: Add Sidecar Atomic Write and Status Update

**Files:**
- Modify: `crates/silica-storage/src/lib.rs`

- [ ] **Step 1: Add failing write tests**

Add these tests:

```rust
#[test]
fn writes_sidecar_under_library_and_updates_status_after_success() {
    let workspace = unique_library_root("sidecar-write");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
    let original_before = std::fs::read(&supported_file).expect("read original before");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        3,
        false,
        true,
        Some("red".to_string()),
    )
    .expect("set flags");

    let result = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write sidecar");

    assert_eq!(result.photo_id, photo_id);
    assert_eq!(result.sidecar_relative_path, format!("sidecars/{photo_id}.silicaraw.sidecar.json"));
    assert!(result.sidecar_path.is_file());
    assert!(result.sidecar_path.starts_with(library.root_path.join(SIDECAR_DIRECTORY)));
    assert!(result.bytes_written > 0);
    assert_eq!(
        std::fs::read(&supported_file).expect("read original after"),
        original_before
    );

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&result.sidecar_path).expect("read sidecar"))
            .expect("parse sidecar");
    validate_sidecar_json(&json).expect("validate written sidecar");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let (sidecar_path, conflict_state): (String, String) = connection
        .query_row(
            "SELECT sidecar_path, conflict_state FROM sidecar_status WHERE photo_id = ?1",
            params![result.photo_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("sidecar status");
    assert_eq!(sidecar_path, result.sidecar_relative_path);
    assert_eq!(conflict_state, "clean");

    remove_library_root(&workspace);
}

#[test]
fn failed_sidecar_write_does_not_replace_existing_valid_sidecar() {
    let workspace = unique_library_root("sidecar-write-failure");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let first = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("first write");
    let first_bytes = std::fs::read(&first.sidecar_path).expect("read first sidecar");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    connection
        .execute(
            "UPDATE photo_flags SET color_label = 'cyan' WHERE photo_id = ?1",
            params![photo_id],
        )
        .expect("force invalid catalog label");
    drop(connection);

    let error = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect_err("invalid write should fail");
    assert!(error.to_string().contains("unsupported sidecar color label"));
    assert_eq!(
        std::fs::read(&first.sidecar_path).expect("read preserved sidecar"),
        first_bytes
    );

    remove_library_root(&workspace);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test -p silica-storage sidecar_write
```

Expected: fails because `write_photo_sidecar` and `SidecarWriteResult` do not exist.

- [ ] **Step 3: Add write result type**

Add this public type near `ExportRecord`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarWriteResult {
    pub photo_id: String,
    pub sidecar_path: PathBuf,
    pub sidecar_relative_path: String,
    pub written_at: String,
    pub bytes_written: u64,
}
```

- [ ] **Step 4: Add public write function**

Add:

```rust
/// Write a validated sidecar into the library-local sidecars directory.
pub fn write_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<SidecarWriteResult, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_local_library(library_root_path)?;
    let sidecar_path = sidecar_path_for_photo(&library.root_path, photo_id)?;
    let sidecar_relative_path = format!("{SIDECAR_DIRECTORY}/{photo_id}.silicaraw.sidecar.json");
    fs::create_dir_all(library.root_path.join(SIDECAR_DIRECTORY))?;

    let value = build_photo_sidecar_value(&library.root_path, photo_id, app_version)?;
    validate_sidecar_json(&value)?;
    let bytes = serde_json::to_vec_pretty(&value)?;
    let temp_path = sidecar_path.with_extension("json.tmp");
    fs::write(&temp_path, &bytes)?;
    let temp_value: serde_json::Value = serde_json::from_slice(&fs::read(&temp_path)?)?;
    validate_sidecar_json(&temp_value)?;
    fs::rename(&temp_path, &sidecar_path)?;

    let written_at = value
        .get("written_at")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    update_sidecar_status(
        &library.catalog_path,
        photo_id,
        &sidecar_relative_path,
        &written_at,
    )?;

    Ok(SidecarWriteResult {
        photo_id: photo_id.to_string(),
        sidecar_path,
        sidecar_relative_path,
        written_at,
        bytes_written: bytes.len() as u64,
    })
}
```

- [ ] **Step 5: Add status update helper**

Add:

```rust
fn update_sidecar_status(
    catalog_path: &Path,
    photo_id: &str,
    sidecar_relative_path: &str,
    written_at: &str,
) -> Result<(), LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    connection.execute(
        r#"
        INSERT INTO sidecar_status(photo_id, sidecar_path, last_written_at, conflict_state)
        VALUES (?1, ?2, ?3, 'clean')
        ON CONFLICT(photo_id) DO UPDATE SET
          sidecar_path = excluded.sidecar_path,
          last_written_at = excluded.last_written_at,
          conflict_state = 'clean'
        "#,
        params![photo_id, sidecar_relative_path, written_at],
    )?;
    Ok(())
}
```

- [ ] **Step 6: Run focused write tests**

Run:

```bash
cargo test -p silica-storage sidecar_write
```

Expected: both write tests pass.

- [ ] **Step 7: Commit Task 4**

Run:

```bash
git add crates/silica-storage/src/lib.rs
git commit -m "feat(storage): write library sidecars"
```

Expected: commit contains atomic sidecar write, sidecar status update, and write tests.

## Task 5: Add Validated Sidecar Read

**Files:**
- Modify: `crates/silica-storage/src/lib.rs`

- [ ] **Step 1: Add failing read tests**

Add these tests:

```rust
#[test]
fn reads_valid_sidecar_without_mutating_catalog_flags() {
    let workspace = unique_library_root("sidecar-read");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set sidecar flags");
    write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write sidecar");
    set_photo_flags(&library.root_path, photo_id.clone(), 1, false, true, None)
        .expect("change catalog flags after write");

    let sidecar = read_photo_sidecar(&library.root_path, &photo_id)
        .expect("read sidecar")
        .expect("sidecar exists");
    assert_eq!(sidecar.photo_id, photo_id);
    assert_eq!(sidecar.flags.rating, 4);
    assert!(sidecar.flags.picked);
    assert!(!sidecar.flags.rejected);
    assert_eq!(sidecar.flags.color_label.as_deref(), Some("green"));
    assert_eq!(sidecar.edit_graph.metadata.rating, 4);

    let live_flags = get_photo_flags(&library.root_path, &sidecar.photo_id)
        .expect("read live flags")
        .expect("live flags");
    assert_eq!(live_flags.rating, 1);
    assert!(!live_flags.picked);
    assert!(live_flags.rejected);
    assert_eq!(live_flags.color_label, None);

    remove_library_root(&workspace);
}

#[test]
fn sidecar_read_rejects_malformed_and_mismatched_payloads() {
    let workspace = unique_library_root("sidecar-read-invalid");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let sidecar_path = sidecar_path_for_photo(&library.root_path, &photo_id)
        .expect("sidecar path");
    std::fs::write(&sidecar_path, b"{not json").expect("write malformed");
    assert!(
        read_photo_sidecar(&library.root_path, &photo_id).is_err(),
        "malformed sidecar must fail"
    );

    write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write valid sidecar");
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sidecar_path).expect("read sidecar"))
            .expect("parse sidecar");
    value["photo"]["photo_id"] = serde_json::Value::String("other-photo".to_string());
    std::fs::write(
        &sidecar_path,
        serde_json::to_vec_pretty(&value).expect("serialize mismatch"),
    )
    .expect("write mismatch");
    let error = read_photo_sidecar(&library.root_path, &photo_id)
        .expect_err("photo id mismatch must fail");
    assert!(error.to_string().contains("sidecar photo id mismatch"));

    remove_library_root(&workspace);
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
cargo test -p silica-storage sidecar_read
```

Expected: fails because `read_photo_sidecar` and `ValidatedSidecar` do not exist.

- [ ] **Step 3: Add validated sidecar type**

Add:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedSidecar {
    pub photo_id: String,
    pub sidecar_path: PathBuf,
    pub written_at: String,
    pub flags: PhotoFlags,
    pub edit_graph: silica_edit::EditGraph,
    pub json: serde_json::Value,
}
```

- [ ] **Step 4: Add public read function**

Add:

```rust
/// Read and validate a library-local sidecar without mutating catalog state.
pub fn read_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ValidatedSidecar>, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let sidecar_path = sidecar_path_for_photo(&library.root_path, photo_id)?;
    if !sidecar_path.is_file() {
        return Ok(None);
    }

    let json: serde_json::Value = serde_json::from_slice(&fs::read(&sidecar_path)?)?;
    validate_sidecar_json(&json)?;
    let sidecar_photo_id = json["photo"]["photo_id"].as_str().ok_or_else(|| {
        LibraryStorageError::SidecarValidation("sidecar.photo.photo_id must be a string".to_string())
    })?;
    if sidecar_photo_id != photo_id {
        return Err(LibraryStorageError::SidecarValidation(format!(
            "sidecar photo id mismatch: expected {photo_id}, found {sidecar_photo_id}"
        )));
    }

    let flags = parse_sidecar_flags(&json)?;
    let edit_graph: silica_edit::EditGraph = serde_json::from_value(json["edit_graph"].clone())?;
    silica_edit::validate_edit_graph(&edit_graph)?;
    let written_at = json["written_at"].as_str().unwrap_or_default().to_string();

    Ok(Some(ValidatedSidecar {
        photo_id: photo_id.to_string(),
        sidecar_path,
        written_at,
        flags,
        edit_graph,
        json,
    }))
}
```

- [ ] **Step 5: Add sidecar flag parser**

Add:

```rust
fn parse_sidecar_flags(value: &serde_json::Value) -> Result<PhotoFlags, LibraryStorageError> {
    let photo_id = value["photo"]["photo_id"]
        .as_str()
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.photo.photo_id must be a string".to_string()))?;
    let flags = value
        .get("flags")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.flags must be an object".to_string()))?;
    let rating = flags
        .get("rating")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.flags.rating must be an integer".to_string()))?;
    let picked = flags
        .get("picked")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.flags.picked must be boolean".to_string()))?;
    let rejected = flags
        .get("rejected")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.flags.rejected must be boolean".to_string()))?;
    let color_label = match flags.get("color_label") {
        Some(value) if value.is_null() => None,
        Some(value) => Some(
            value
                .as_str()
                .ok_or_else(|| LibraryStorageError::SidecarValidation("sidecar.flags.color_label must be string or null".to_string()))?
                .to_string(),
        ),
        None => None,
    };

    PhotoFlags::new(photo_id.to_string(), rating as u8, picked, rejected, color_label)
        .map_err(LibraryStorageError::from)
}
```

- [ ] **Step 6: Run focused read tests**

Run:

```bash
cargo test -p silica-storage sidecar_read
```

Expected: both read tests pass.

- [ ] **Step 7: Commit Task 5**

Run:

```bash
git add crates/silica-storage/src/lib.rs
git commit -m "feat(storage): read validated sidecars"
```

Expected: commit contains validated sidecar read behavior and read tests.

## Task 6: Add Core Sidecar Wrappers

**Files:**
- Modify: `crates/silica-core/src/lib.rs`

- [ ] **Step 1: Add failing core test**

Add this test in `crates/silica-core/src/lib.rs`:

```rust
#[test]
fn writes_and_reads_photo_sidecar_through_core() {
    let workspace = unique_library_root("core-sidecar");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    write_source_jpeg(&jpeg_file);
    let original_hash = file_hash(&jpeg_file);

    let created = create_library(&library_root).expect("create library");
    import_folder(&created.root_path, &import_root).expect("import folder");
    let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);
    set_photo_flags(
        &created.root_path,
        photo_id.clone(),
        2,
        true,
        false,
        Some("blue".to_string()),
    )
    .expect("set flags");

    let written = write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write sidecar")
        .expect("sidecar write result");
    assert_eq!(written.photo_id, photo_id);
    assert!(written.sidecar_path.is_file());
    assert_original_hash(&jpeg_file, &original_hash, "core sidecar write");

    let read = read_photo_sidecar(&created.root_path, &photo_id)
        .expect("read sidecar")
        .expect("sidecar exists");
    assert_eq!(read.photo_id, photo_id);
    assert_eq!(read.flags.rating, 2);
    assert_eq!(read.flags.color_label.as_deref(), Some("blue"));
    assert_original_hash(&jpeg_file, &original_hash, "core sidecar read");

    remove_library_root(&workspace);
}
```

- [ ] **Step 2: Run test and verify failure**

Run:

```bash
cargo test -p silica-core sidecar
```

Expected: fails because core wrappers do not exist.

- [ ] **Step 3: Re-export sidecar types**

Near existing core re-exports, add:

```rust
pub use silica_storage::SidecarWriteResult;
pub use silica_storage::ValidatedSidecar;
```

- [ ] **Step 4: Add wrapper functions**

Add these functions near the flag/edit APIs:

```rust
/// Write a library-local sidecar through the core command boundary.
pub fn write_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<Option<SidecarWriteResult>, CoreError> {
    match silica_storage::write_photo_sidecar(library_root_path, photo_id, app_version) {
        Ok(result) => Ok(Some(result)),
        Err(silica_storage::LibraryStorageError::MissingPhoto(_)) => Ok(None),
        Err(error) => Err(CoreError::from(error)),
    }
}

/// Read a validated library-local sidecar through the core command boundary.
pub fn read_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ValidatedSidecar>, CoreError> {
    silica_storage::read_photo_sidecar(library_root_path, photo_id).map_err(CoreError::from)
}
```

- [ ] **Step 5: Run focused core test**

Run:

```bash
cargo test -p silica-core sidecar
```

Expected: the core sidecar test passes.

- [ ] **Step 6: Commit Task 6**

Run:

```bash
git add crates/silica-core/src/lib.rs
git commit -m "feat(core): expose sidecar workflows"
```

Expected: commit contains only core wrappers, re-exports, and core test.

## Task 7: Update Docs for Task 10.3 Completion

**Files:**
- Modify: `crates/silica-storage/README.md`
- Modify: `crates/silica-core/README.md`
- Modify: `docs/wiki/topics/catalog.md`
- Modify: `docs/wiki/topics/data-safety.md`
- Modify: `docs/wiki/roadmaps/post-alpha-product-roadmap.md`
- Modify: `docs/wiki/log.md`

- [ ] **Step 1: Update storage README**

Replace the first paragraph that says storage does not write sidecars with:

```markdown
This crate currently owns the catalog migration runner, initial empty catalog schema/index proof, local library create/open, Phase 4.3 folder import scanner, Phase 4.4 photo flags persistence, Phase 5.3 active edit graph commit/read behavior, and Task 10.3 library-local sidecar read/write behavior. It does not decode photos, extract camera metadata, mutate originals, write sidecars next to originals, manage automatic sidecar sync, or expose database access to plugins/MCP.
```

Append:

```markdown
Task 10.3 adds explicit sidecar write/read APIs. Sidecars are written only under `sidecars/` inside the library root, validate the sidecar and nested edit graph payloads, mirror rating/picked/rejected/color-label state only, update `sidecar_status` after successful writes, and do not mutate original referenced files.
```

- [ ] **Step 2: Update core README**

Replace the last paragraph with:

```markdown
Core delegates SQLite, filesystem details, and sidecar JSON validation to `silica-storage`. It does not decode RAW files, render pixels, write sidecars next to originals, expose plugins/MCP, run MLX behavior, or perform automatic sidecar sync.
```

Append:

```markdown
Task 10.3 adds thin sidecar workflow wrappers for explicit sidecar write/read. Core does not duplicate sidecar path or schema logic.
```

- [ ] **Step 3: Update catalog wiki**

In `docs/wiki/topics/catalog.md`, add a Current Stance bullet:

```markdown
- Task 10.3 adds explicit library-local sidecar write/read behavior through `silica-storage`; `photo_flags` remains the live in-app authority until a later explicit sync task changes that policy.
```

Remove these two bullets from Not Implemented Yet:

```markdown
- Sidecar read/write and conflict handling.
- Sidecar flag mirroring.
```

Add these Not Implemented Yet bullets:

```markdown
- Automatic sidecar synchronization.
- Catalog rebuild from sidecars.
- Sidecar conflict handling and conflict UI.
```

- [ ] **Step 4: Update data safety wiki**

In `docs/wiki/topics/data-safety.md`, add:

```markdown
- Task 10.3 writes sidecars only under the library `sidecars/` directory, validates sidecar and nested edit graph JSON, updates `sidecar_status` only after a successful write, and verifies original referenced files remain unchanged.
```

Replace `Sidecar read/write.` in Early Required Tests with:

```markdown
- Sidecar read/write safety. Task 10.3 covers library-local paths, schema-aware validation, status update after success, malformed/mismatched read rejection, and original hash preservation.
```

- [ ] **Step 5: Mark roadmap task complete**

In `docs/wiki/roadmaps/post-alpha-product-roadmap.md`, add this status to Task 10.3:

```markdown
- **Status:** Completed on 2026-06-11. Added explicit library-local sidecar v1 read/write behavior for catalog photo state, active/default edit graph payloads, portable rating/picked/rejected/color-label flags, nested edit graph validation, atomic file writes, validated reads, and `sidecar_status` updates after successful writes. This does not add automatic sidecar sync, catalog rebuild, backup/restore, conflict UI, RAW decoding, color proof, or export proof.
```

Add this validation bullet:

```markdown
  - `python3 scripts/harness/check-sidecar-contract.py`
```

- [ ] **Step 6: Add wiki log entry**

Add this entry at the top of `docs/wiki/log.md` entries:

```markdown
## [2026-06-11] phase-10 | Sidecar v1 foundation added

- Added Task 10.3 sidecar v1 read/write foundation for library-local sidecars.
- Validated sidecar and nested edit graph payloads while keeping `photo_flags` catalog-authoritative during normal app operation.
- Preserved original-file safety and kept automatic sync, rebuild, backup/restore, conflict UI, RAW/color proof, and export proof out of scope.
```

- [ ] **Step 7: Run Markdown links**

Run:

```bash
python3 scripts/harness/check-md-links.py
```

Expected: local links ok.

- [ ] **Step 8: Commit Task 7**

Run:

```bash
git add crates/silica-storage/README.md crates/silica-core/README.md docs/wiki/topics/catalog.md docs/wiki/topics/data-safety.md docs/wiki/roadmaps/post-alpha-product-roadmap.md docs/wiki/log.md
git commit -m "docs(sidecars): document sidecar v1 foundation"
```

Expected: commit contains only documentation updates.

## Task 8: Final Verification and PR

**Files:**
- All Task 10.3 files

- [ ] **Step 1: Format Rust**

Run:

```bash
cargo fmt --all
```

Expected: command exits with status 0.

- [ ] **Step 2: Run focused tests**

Run:

```bash
cargo test -p silica-storage sidecar
cargo test -p silica-core sidecar
python3 scripts/harness/check-sidecar-contract.py
```

Expected: all commands exit with status 0.

- [ ] **Step 3: Run full harness**

Run:

```bash
scripts/harness/check.sh
```

Expected: command exits with status 0 and prints `Harness checks passed`.

- [ ] **Step 4: Run code-review graph**

Run code-review graph change detection against `main` for:

```txt
crates/silica-storage/src/lib.rs
crates/silica-core/src/lib.rs
scripts/harness/check-sidecar-contract.py
scripts/harness/check.sh
crates/silica-storage/README.md
crates/silica-core/README.md
docs/wiki/topics/catalog.md
docs/wiki/topics/data-safety.md
docs/wiki/roadmaps/post-alpha-product-roadmap.md
docs/wiki/log.md
```

Expected: review output reports no unreviewed high-risk flow or test gap. If it reports a real issue, fix it before opening the PR.

- [ ] **Step 5: Inspect final diff**

Run:

```bash
git status -sb
git diff --stat main
```

Expected: only Task 10.3 implementation, harness, and docs files are changed.

- [ ] **Step 6: Open PR**

Run:

```bash
git push -u origin feature/phase-10-sidecar-v1
gh pr create --title "[codex] Implement Sidecar v1 read/write foundation" --body-file /tmp/silicaraw-task-10-3-pr.md --base main --head feature/phase-10-sidecar-v1
```

Expected: PR opens with validation results and explicit exclusions.

## Non-Goals

Do not add:

- RAW decoding
- Core Image probing
- LibRaw
- ICC parsing
- golden image comparison
- pixel rendering proof
- export proof fields
- automatic sidecar sync
- next-to-original sidecars
- catalog rebuild from sidecars
- backup or restore behavior
- conflict UI
- MLX
- MCP
- plugin runtime
- cloud sync
- telemetry
- auto-update
- Homebrew distribution
- new external dependencies

## Self-Review Checklist

- Task 10.3 writes sidecars only under the library `sidecars/` directory.
- `sidecar.flags` contains exactly rating, picked, rejected, and color label.
- `edited`, `exported`, and export history remain out of sidecar flags.
- Building a default edit graph for a sidecar does not insert an `edit_states` row.
- Sidecar reads do not mutate catalog flags.
- Invalid color labels fail before sidecar replacement.
- Original referenced files are hash-checked in storage and core tests.
- Docs state that rebuild, backup/restore, conflict UI, RAW proof, color proof, and export proof remain later work.
- `scripts/harness/check.sh` includes the sidecar contract guard.
