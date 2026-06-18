//! MCP boundary for SilicaRAW.
//!
//! Task 23.1 records permission ID boundaries only.

use std::error::Error;
use std::fmt;

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

#[cfg(test)]
mod tests {
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
}
