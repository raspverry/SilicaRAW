//! MCP boundary for SilicaRAW.
//!
//! Task 23.1 records permission ID boundaries only.

use std::error::Error;
use std::fmt;
use std::path::Path;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-mcp";

pub const MCP_MODE_PERMISSION_IDS: &[&str] =
    &["mcp:read_only", "mcp:review", "mcp:edit", "mcp:export"];

pub const MCP_DEFAULT_GRANTED_PERMISSION_IDS: &[&str] = &[];

const MCP_TOOL_MANIFEST_SCHEMA: &str = "silica.mcp_tool";
const MCP_TOOL_MANIFEST_VERSION: i64 = 1;
const MCP_TOOL_MANIFEST_PERMISSION: &str = "mcp:read_only";

const MCP_TOOL_MANIFEST_REQUIRED_FIELDS: &[&str] = &[
    "schema",
    "version",
    "tool_id",
    "name",
    "description",
    "permission",
    "requires_confirmation",
    "side_effects",
    "undoable",
    "input_schema",
    "output_schema",
];

const MCP_TOOL_MANIFEST_ALLOWED_FIELDS: &[&str] = MCP_TOOL_MANIFEST_REQUIRED_FIELDS;

pub const READ_ONLY_MCP_TOOL_IDS: &[&str] = &[
    "silica.photos.list",
    "silica.photos.get",
    "silica.photos.get_metadata",
    "silica.collections.list",
    "silica.selection.get",
    "silica.presets.list",
    "silica.exports.list",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum McpBoundaryMode {
    Off,
    ReadOnly,
    Review,
    Edit,
    Export,
}

pub fn permission_id_for_mcp_mode(mode: McpBoundaryMode) -> Option<&'static str> {
    match mode {
        McpBoundaryMode::Off => None,
        McpBoundaryMode::ReadOnly => Some("mcp:read_only"),
        McpBoundaryMode::Review => Some("mcp:review"),
        McpBoundaryMode::Edit => Some("mcp:edit"),
        McpBoundaryMode::Export => Some("mcp:export"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpToolManifest {
    pub tool_id: String,
    pub name: String,
    pub description: String,
    pub permission: String,
    pub requires_confirmation: bool,
    pub side_effects: Vec<String>,
    pub undoable: bool,
    pub direct_sqlite_access: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpToolManifestError {
    InvalidJson(String),
    InvalidField(String),
    ForbiddenTool(String),
}

impl fmt::Display for McpToolManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => {
                write!(formatter, "invalid MCP tool manifest JSON: {message}")
            }
            Self::InvalidField(message) => {
                write!(formatter, "invalid MCP tool manifest field: {message}")
            }
            Self::ForbiddenTool(tool_id) => write!(formatter, "forbidden MCP tool: {tool_id}"),
        }
    }
}

impl Error for McpToolManifestError {}

pub fn validate_mcp_tool_manifest_json(
    manifest_json: &str,
) -> Result<McpToolManifest, McpToolManifestError> {
    let value: serde_json::Value = serde_json::from_str(manifest_json)
        .map_err(|error| McpToolManifestError::InvalidJson(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        McpToolManifestError::InvalidField("manifest root must be an object".to_string())
    })?;

    for key in object.keys() {
        if !MCP_TOOL_MANIFEST_ALLOWED_FIELDS.contains(&key.as_str()) {
            return Err(McpToolManifestError::InvalidField(format!(
                "unknown field {key}"
            )));
        }
    }
    for field in MCP_TOOL_MANIFEST_REQUIRED_FIELDS {
        if !object.contains_key(*field) {
            return Err(McpToolManifestError::InvalidField(format!(
                "missing {field}"
            )));
        }
    }

    let schema = required_string(object, "schema")?;
    if schema != MCP_TOOL_MANIFEST_SCHEMA {
        return Err(McpToolManifestError::InvalidField(format!(
            "schema must be {MCP_TOOL_MANIFEST_SCHEMA}"
        )));
    }
    let version = object
        .get("version")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| McpToolManifestError::InvalidField("version must be 1".to_string()))?;
    if version != MCP_TOOL_MANIFEST_VERSION {
        return Err(McpToolManifestError::InvalidField(format!(
            "version must be {MCP_TOOL_MANIFEST_VERSION}"
        )));
    }

    let tool_id = required_string(object, "tool_id")?;
    if !READ_ONLY_MCP_TOOL_IDS.contains(&tool_id.as_str()) {
        return Err(McpToolManifestError::ForbiddenTool(tool_id));
    }

    let permission = required_string(object, "permission")?;
    if permission != MCP_TOOL_MANIFEST_PERMISSION {
        return Err(McpToolManifestError::InvalidField(format!(
            "permission must be {MCP_TOOL_MANIFEST_PERMISSION}"
        )));
    }
    let requires_confirmation = required_bool(object, "requires_confirmation")?;
    if requires_confirmation {
        return Err(McpToolManifestError::InvalidField(
            "requires_confirmation must be false for read-only tools".to_string(),
        ));
    }
    let side_effects = required_empty_string_array(object, "side_effects")?;
    let undoable = required_bool(object, "undoable")?;
    if undoable {
        return Err(McpToolManifestError::InvalidField(
            "undoable must be false for read-only tools".to_string(),
        ));
    }
    required_object(object, "input_schema")?;
    required_object(object, "output_schema")?;

    Ok(McpToolManifest {
        tool_id,
        name: required_string(object, "name")?,
        description: required_string(object, "description")?,
        permission,
        requires_confirmation,
        side_effects,
        undoable,
        direct_sqlite_access: false,
    })
}

fn required_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, McpToolManifestError> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| McpToolManifestError::InvalidField(format!("{field} must be a string")))?;
    if value.trim().is_empty() {
        return Err(McpToolManifestError::InvalidField(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value.to_string())
}

fn required_bool(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<bool, McpToolManifestError> {
    object
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| McpToolManifestError::InvalidField(format!("{field} must be a boolean")))
}

fn required_empty_string_array(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<Vec<String>, McpToolManifestError> {
    let values = object
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| McpToolManifestError::InvalidField(format!("{field} must be an array")))?;
    if !values.is_empty() {
        return Err(McpToolManifestError::InvalidField(format!(
            "{field} must be empty for read-only tools"
        )));
    }
    Ok(Vec::new())
}

fn required_object(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<(), McpToolManifestError> {
    object
        .get(field)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| McpToolManifestError::InvalidField(format!("{field} must be an object")))?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpReadOnlyToolInput {
    pub photo_id: Option<String>,
    pub offset: u64,
    pub limit: u16,
}

impl Default for McpReadOnlyToolInput {
    fn default() -> Self {
        Self {
            photo_id: None,
            offset: 0,
            limit: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpReadOnlyToolResult {
    pub tool_id: String,
    pub payload_json: String,
    pub action_log_id: String,
}

#[derive(Debug)]
pub enum McpReadOnlyToolError {
    Manifest(McpToolManifestError),
    Core(silica_core::CoreError),
    InvalidInput(String),
}

impl fmt::Display for McpReadOnlyToolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "{error}"),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::InvalidInput(message) => write!(formatter, "invalid MCP tool input: {message}"),
        }
    }
}

impl Error for McpReadOnlyToolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Manifest(error) => Some(error),
            Self::Core(error) => Some(error),
            Self::InvalidInput(_) => None,
        }
    }
}

impl From<McpToolManifestError> for McpReadOnlyToolError {
    fn from(error: McpToolManifestError) -> Self {
        Self::Manifest(error)
    }
}

impl From<silica_core::CoreError> for McpReadOnlyToolError {
    fn from(error: silica_core::CoreError) -> Self {
        Self::Core(error)
    }
}

pub fn run_read_only_mcp_tool(
    manifest_json: &str,
    library_root_path: impl AsRef<Path>,
    session_id: impl AsRef<str>,
    input: McpReadOnlyToolInput,
) -> Result<McpReadOnlyToolResult, McpReadOnlyToolError> {
    let manifest = validate_mcp_tool_manifest_json(manifest_json)?;
    let library_root_path = library_root_path.as_ref();
    let session_id = session_id.as_ref();

    let (payload, subject_type, subject_id) = match manifest.tool_id.as_str() {
        "silica.photos.list" => (
            photos_list_payload(library_root_path, input.offset, input.limit)?,
            "catalog",
            None,
        ),
        "silica.photos.get" => {
            let photo_id = required_photo_id(&input)?;
            (
                photo_get_payload(library_root_path, photo_id)?,
                "photo",
                Some(photo_id.to_string()),
            )
        }
        "silica.photos.get_metadata" => {
            let photo_id = required_photo_id(&input)?;
            (
                photo_metadata_payload(library_root_path, photo_id)?,
                "photo_metadata",
                Some(photo_id.to_string()),
            )
        }
        "silica.collections.list" => (
            serde_json::json!({
                "collections": [],
                "source": "collections_core_api_not_available"
            }),
            "collections",
            None,
        ),
        "silica.selection.get" => (
            serde_json::json!({
                "selected_photo_id": null,
                "source": "library_selection_not_persisted"
            }),
            "selection",
            None,
        ),
        "silica.presets.list" => (presets_payload(library_root_path)?, "presets", None),
        "silica.exports.list" => (
            exports_payload(library_root_path, input.limit)?,
            "exports",
            None,
        ),
        other => {
            return Err(McpReadOnlyToolError::InvalidInput(format!(
                "unsupported tool_id {other}"
            )));
        }
    };

    let action_log = silica_core::record_mcp_read(
        library_root_path,
        session_id,
        subject_type,
        subject_id.as_deref(),
        silica_core::ExtensionPermission::McpReadOnly,
    )?;

    Ok(McpReadOnlyToolResult {
        tool_id: manifest.tool_id,
        payload_json: payload.to_string(),
        action_log_id: action_log.id,
    })
}

fn required_photo_id(input: &McpReadOnlyToolInput) -> Result<&str, McpReadOnlyToolError> {
    input
        .photo_id
        .as_deref()
        .filter(|photo_id| !photo_id.trim().is_empty())
        .ok_or_else(|| McpReadOnlyToolError::InvalidInput("photo_id is required".to_string()))
}

fn photos_list_payload(
    library_root_path: &Path,
    offset: u64,
    limit: u16,
) -> Result<serde_json::Value, McpReadOnlyToolError> {
    let page = silica_core::query_library_photos(
        library_root_path,
        silica_core::LibraryQueryRequest::new(
            offset,
            limit,
            silica_core::LibraryQuerySort::ImportedAtDesc,
            silica_core::LibraryQueryFilters::default(),
        ),
    )?;
    Ok(serde_json::json!({
        "photos": page.items.iter().map(photo_grid_item_json).collect::<Vec<_>>(),
        "offset": page.offset,
        "limit": page.limit,
        "total_count": page.total_count,
        "has_next_page": page.has_next_page,
    }))
}

fn photo_get_payload(
    library_root_path: &Path,
    photo_id: &str,
) -> Result<serde_json::Value, McpReadOnlyToolError> {
    let photo = silica_core::get_mcp_photo_read_record(library_root_path, photo_id)?;
    Ok(serde_json::json!({
        "photo": photo.as_ref().map(mcp_photo_read_record_json),
    }))
}

fn photo_metadata_payload(
    library_root_path: &Path,
    photo_id: &str,
) -> Result<serde_json::Value, McpReadOnlyToolError> {
    let metadata = silica_core::get_photo_metadata(library_root_path, photo_id)?;
    Ok(serde_json::json!({
        "metadata": metadata.as_ref().map(photo_metadata_json),
    }))
}

fn exports_payload(
    library_root_path: &Path,
    limit: u16,
) -> Result<serde_json::Value, McpReadOnlyToolError> {
    let exports = silica_core::list_recent_exports(library_root_path, usize::from(limit))?;
    Ok(serde_json::json!({
        "exports": exports.iter().map(recent_export_json).collect::<Vec<_>>(),
    }))
}

fn presets_payload(library_root_path: &Path) -> Result<serde_json::Value, McpReadOnlyToolError> {
    let catalog = silica_core::get_export_settings_catalog(library_root_path)?;
    Ok(serde_json::json!({
        "default_preset_id": catalog.default_preset_id,
        "default_settings": export_settings_json(&catalog.default_settings),
        "export_presets": catalog.presets.iter().map(export_preset_json).collect::<Vec<_>>(),
    }))
}

fn photo_grid_item_json(photo: &silica_core::LibraryPhotoGridItem) -> serde_json::Value {
    serde_json::json!({
        "photo_id": photo.photo_id,
        "file_name": photo.file_name,
        "path": photo.path,
        "file_type": photo.file_type,
        "thumbnail_path": photo.thumbnail_path,
        "missing": photo.missing,
        "unsupported": photo.unsupported,
        "rating": photo.rating,
        "picked": photo.picked,
        "rejected": photo.rejected,
        "color_label": photo.color_label,
    })
}

fn mcp_photo_read_record_json(photo: &silica_core::McpPhotoReadRecord) -> serde_json::Value {
    serde_json::json!({
        "photo_id": photo.photo_id,
        "file_name": photo.file_name,
        "path": photo.path,
        "unsupported": photo.unsupported,
        "rating": photo.rating,
        "picked": photo.picked,
        "rejected": photo.rejected,
        "color_label": photo.color_label,
    })
}

fn photo_metadata_json(metadata: &silica_core::PhotoMetadata) -> serde_json::Value {
    serde_json::json!({
        "photo_id": metadata.photo_id,
        "file_name": metadata.file_name,
        "source_path": metadata.source_path,
        "file_type": metadata.file_type,
        "unsupported": metadata.unsupported,
        "file_size": numeric_metadata_field_json(&metadata.file_size),
        "modified_at": string_metadata_field_json(&metadata.modified_at),
        "width": numeric_metadata_field_json(&metadata.width),
        "height": numeric_metadata_field_json(&metadata.height),
        "orientation": string_metadata_field_json(&metadata.orientation),
        "capture_time": string_metadata_field_json(&metadata.capture_time),
        "camera_make": string_metadata_field_json(&metadata.camera_make),
        "camera_model": string_metadata_field_json(&metadata.camera_model),
        "lens_model": string_metadata_field_json(&metadata.lens_model),
    })
}

fn numeric_metadata_field_json(field: &silica_core::PhotoMetadataField<i64>) -> serde_json::Value {
    serde_json::json!({
        "state": metadata_field_state_string(field.state),
        "value": field.value,
    })
}

fn string_metadata_field_json(
    field: &silica_core::PhotoMetadataField<String>,
) -> serde_json::Value {
    serde_json::json!({
        "state": metadata_field_state_string(field.state),
        "value": field.value,
    })
}

fn metadata_field_state_string(state: silica_core::PhotoMetadataFieldState) -> &'static str {
    match state {
        silica_core::PhotoMetadataFieldState::Known => "known",
        silica_core::PhotoMetadataFieldState::Unknown => "unknown",
        silica_core::PhotoMetadataFieldState::Unavailable => "unavailable",
    }
}

fn recent_export_json(export: &silica_core::PhotoRecentExport) -> serde_json::Value {
    serde_json::json!({
        "export_record_id": export.export_record_id,
        "photo_id": export.photo_id,
        "output_path": export.output_path,
        "export_settings_json": export.export_settings_json,
        "created_at": export.created_at,
        "output_exists": export.output_exists,
    })
}

fn export_preset_json(preset: &silica_core::ExportPreset) -> serde_json::Value {
    serde_json::json!({
        "id": preset.id,
        "name": preset.name,
        "settings": export_settings_json(&preset.settings),
    })
}

fn export_settings_json(settings: &silica_core::ExportSettings) -> serde_json::Value {
    serde_json::json!({
        "format": settings.format,
        "color_profile": settings.color_profile,
        "quality": settings.quality,
        "metadata_policy": settings.metadata_policy,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const VALID_READ_ONLY_TOOL_MANIFEST: &str = r#"{
        "schema": "silica.mcp_tool",
        "version": 1,
        "tool_id": "silica.photos.list",
        "name": "List Photos",
        "description": "List catalog photo records through Core APIs.",
        "permission": "mcp:read_only",
        "requires_confirmation": false,
        "side_effects": [],
        "undoable": false,
        "input_schema": {"type": "object", "additionalProperties": false},
        "output_schema": {"type": "object"}
    }"#;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-mcp");
    }

    #[test]
    fn mcp_modes_require_permission_ids_and_start_denied() {
        assert!(super::MCP_DEFAULT_GRANTED_PERMISSION_IDS.is_empty());

        assert_eq!(
            super::permission_id_for_mcp_mode(super::McpBoundaryMode::Off),
            None
        );
        assert_eq!(
            super::permission_id_for_mcp_mode(super::McpBoundaryMode::ReadOnly),
            Some("mcp:read_only")
        );
    }

    #[test]
    fn validates_read_only_mcp_tool_manifest() {
        let manifest = super::validate_mcp_tool_manifest_json(VALID_READ_ONLY_TOOL_MANIFEST)
            .expect("read-only manifest should validate");

        assert_eq!(manifest.tool_id, "silica.photos.list");
        assert_eq!(manifest.permission, "mcp:read_only");
        assert_eq!(manifest.side_effects, Vec::<String>::new());
        assert!(!manifest.requires_confirmation);
        assert!(!manifest.undoable);
        assert!(!manifest.direct_sqlite_access);
    }

    #[test]
    fn validates_each_allowed_read_only_tool_id() {
        for tool_id in super::READ_ONLY_MCP_TOOL_IDS {
            let manifest_json =
                VALID_READ_ONLY_TOOL_MANIFEST.replace("silica.photos.list", tool_id);
            let manifest = super::validate_mcp_tool_manifest_json(&manifest_json)
                .expect("allowed read-only tool should validate");
            assert_eq!(manifest.tool_id, *tool_id);
        }
    }

    #[test]
    fn rejects_non_read_only_mcp_tool_manifests() {
        let mutating_tool = VALID_READ_ONLY_TOOL_MANIFEST
            .replace("silica.photos.list", "silica.photos.set_rating")
            .replace("mcp:read_only", "mcp:edit");
        assert!(super::validate_mcp_tool_manifest_json(&mutating_tool).is_err());

        let with_side_effect = VALID_READ_ONLY_TOOL_MANIFEST.replace(
            "\"side_effects\": []",
            "\"side_effects\": [\"catalog_write\"]",
        );
        assert!(super::validate_mcp_tool_manifest_json(&with_side_effect).is_err());

        let with_direct_sqlite = VALID_READ_ONLY_TOOL_MANIFEST.replace(
            "\"output_schema\": {\"type\": \"object\"}",
            "\"output_schema\": {\"type\": \"object\"}, \"direct_sqlite_access\": true",
        );
        assert!(super::validate_mcp_tool_manifest_json(&with_direct_sqlite).is_err());
    }

    #[test]
    fn read_only_adapter_lists_photos_through_core_and_logs_read() {
        let workspace = unique_test_root("mcp-adapter-list");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        std::fs::create_dir_all(&import_root).expect("create import root");
        std::fs::write(import_root.join("sample.DNG"), b"supported raw candidate")
            .expect("write sample");

        let created = silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&created.root_path, &import_root).expect("import folder");

        let result = super::run_read_only_mcp_tool(
            VALID_READ_ONLY_TOOL_MANIFEST,
            &created.root_path,
            "session-26-3",
            super::McpReadOnlyToolInput::default(),
        )
        .expect("run list tool");

        assert_eq!(result.tool_id, "silica.photos.list");
        assert!(result.payload_json.contains("sample.DNG"));
        assert!(result.payload_json.contains("\"photos\""));
        assert!(!result.action_log_id.is_empty());

        let entries =
            silica_core::list_action_log_entries(&created.root_path, 10).expect("action log");
        assert!(entries.iter().any(|entry| entry.action_type == "mcp_read"
            && entry.actor_type == "mcp"
            && entry.actor_id.as_deref() == Some("session-26-3")
            && entry.subject_type.as_deref() == Some("catalog")));

        remove_test_root(&workspace);
    }

    #[test]
    fn read_only_adapter_reads_metadata_and_recent_exports_without_mutation_tool() {
        let workspace = unique_test_root("mcp-adapter-metadata");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        std::fs::create_dir_all(&import_root).expect("create import root");
        std::fs::write(import_root.join("sample.jpg"), b"not a real jpeg").expect("write sample");

        let created = silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&created.root_path, &import_root).expect("import folder");
        let photo_id = silica_core::query_library_photos(
            &created.root_path,
            silica_core::LibraryQueryRequest::default(),
        )
        .expect("query photos")
        .items
        .first()
        .expect("photo row")
        .photo_id
        .clone();

        let metadata_manifest = VALID_READ_ONLY_TOOL_MANIFEST
            .replace("silica.photos.list", "silica.photos.get_metadata");
        let metadata = super::run_read_only_mcp_tool(
            &metadata_manifest,
            &created.root_path,
            "session-26-3",
            super::McpReadOnlyToolInput {
                photo_id: Some(photo_id.clone()),
                ..super::McpReadOnlyToolInput::default()
            },
        )
        .expect("run metadata tool");
        assert!(metadata.payload_json.contains(&photo_id));
        assert!(metadata.payload_json.contains("\"metadata\""));

        let get_manifest =
            VALID_READ_ONLY_TOOL_MANIFEST.replace("silica.photos.list", "silica.photos.get");
        let photo = super::run_read_only_mcp_tool(
            &get_manifest,
            &created.root_path,
            "session-26-3",
            super::McpReadOnlyToolInput {
                photo_id: Some(photo_id.clone()),
                ..super::McpReadOnlyToolInput::default()
            },
        )
        .expect("run photo get tool");
        assert!(photo.payload_json.contains("sample.jpg"));
        assert!(photo.payload_json.contains("\"photo\""));

        let exports_manifest =
            VALID_READ_ONLY_TOOL_MANIFEST.replace("silica.photos.list", "silica.exports.list");
        let exports = super::run_read_only_mcp_tool(
            &exports_manifest,
            &created.root_path,
            "session-26-3",
            super::McpReadOnlyToolInput::default(),
        )
        .expect("run exports tool");
        assert!(exports.payload_json.contains("\"exports\":[]"));

        assert!(!super::READ_ONLY_MCP_TOOL_IDS.contains(&"silica.photos.set_rating"));

        remove_test_root(&workspace);
    }

    #[test]
    fn read_only_adapter_rejects_photo_tool_without_photo_id() {
        let workspace = unique_test_root("mcp-adapter-missing-photo");
        let library_root = workspace.join("SilicaRAW Library");
        let created = silica_core::create_library(&library_root).expect("create library");
        let metadata_manifest = VALID_READ_ONLY_TOOL_MANIFEST
            .replace("silica.photos.list", "silica.photos.get_metadata");

        let error = super::run_read_only_mcp_tool(
            &metadata_manifest,
            &created.root_path,
            "session-26-3",
            super::McpReadOnlyToolInput::default(),
        )
        .expect_err("missing photo_id should fail");

        assert!(matches!(
            error,
            super::McpReadOnlyToolError::InvalidInput(_)
        ));

        remove_test_root(&workspace);
    }

    fn unique_test_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("silica-raw-{label}-{nonce}"))
    }

    fn remove_test_root(path: &PathBuf) {
        if let Err(error) = std::fs::remove_dir_all(path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic!("remove test root: {error}");
            }
        }
    }
}
