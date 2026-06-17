//! Plugin boundary for SilicaRAW.
//!
//! Task 23.1 records permission ID boundaries only.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-plugin";

pub const PLUGIN_BOUNDARY_PERMISSION_IDS: &[&str] = &[
    "metadata:read",
    "edit_suggestion:read",
    "edit_suggestion:apply",
    "export:local",
    "filesystem:limited_read",
    "ai_result:read",
];

pub const PLUGIN_DEFAULT_GRANTED_PERMISSION_IDS: &[&str] = &[];

const PLUGIN_MANIFEST_SCHEMA: &str = "silica.plugin";
const PLUGIN_MANIFEST_VERSION: i64 = 1;

const PLUGIN_MANIFEST_REQUIRED_FIELDS: &[&str] = &[
    "schema",
    "version",
    "plugin_id",
    "name",
    "description",
    "author",
    "license",
    "plugin_version",
    "minimum_silica_version",
    "type",
    "permissions",
];

const PLUGIN_MANIFEST_ALLOWED_FIELDS: &[&str] = &[
    "schema",
    "version",
    "plugin_id",
    "name",
    "description",
    "author",
    "license",
    "plugin_version",
    "minimum_silica_version",
    "type",
    "permissions",
    "entry",
    "homepage",
    "repository",
];

const PLUGIN_MANIFEST_ALLOWED_TYPES: &[&str] =
    &["preset_pack", "export_preset", "ai_model", "workflow"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub plugin_version: String,
    pub minimum_silica_version: String,
    pub plugin_type: String,
    pub permissions: Vec<String>,
    pub entry: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginManifestError {
    InvalidJson(String),
    InvalidField(String),
    ForbiddenPermission(String),
}

impl fmt::Display for PluginManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => {
                write!(formatter, "invalid plugin manifest JSON: {message}")
            }
            Self::InvalidField(message) => {
                write!(formatter, "invalid plugin manifest field: {message}")
            }
            Self::ForbiddenPermission(permission) => {
                write!(formatter, "forbidden plugin permission: {permission}")
            }
        }
    }
}

impl Error for PluginManifestError {}

pub fn validate_plugin_manifest_json(
    manifest_json: &str,
) -> Result<PluginManifest, PluginManifestError> {
    let value: serde_json::Value = serde_json::from_str(manifest_json)
        .map_err(|error| PluginManifestError::InvalidJson(error.to_string()))?;
    let object = value.as_object().ok_or_else(|| {
        PluginManifestError::InvalidField("manifest root must be an object".to_string())
    })?;

    for key in object.keys() {
        if !PLUGIN_MANIFEST_ALLOWED_FIELDS.contains(&key.as_str()) {
            return Err(PluginManifestError::InvalidField(format!(
                "unknown field {key}"
            )));
        }
    }
    for field in PLUGIN_MANIFEST_REQUIRED_FIELDS {
        if !object.contains_key(*field) {
            return Err(PluginManifestError::InvalidField(format!(
                "missing {field}"
            )));
        }
    }

    let schema = required_string(object, "schema")?;
    if schema != PLUGIN_MANIFEST_SCHEMA {
        return Err(PluginManifestError::InvalidField(format!(
            "schema must be {PLUGIN_MANIFEST_SCHEMA}"
        )));
    }
    let version = object
        .get("version")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| PluginManifestError::InvalidField("version must be 1".to_string()))?;
    if version != PLUGIN_MANIFEST_VERSION {
        return Err(PluginManifestError::InvalidField(format!(
            "version must be {PLUGIN_MANIFEST_VERSION}"
        )));
    }

    let plugin_type = required_string(object, "type")?;
    if !PLUGIN_MANIFEST_ALLOWED_TYPES.contains(&plugin_type.as_str()) {
        return Err(PluginManifestError::InvalidField(format!(
            "unsupported type {plugin_type}"
        )));
    }
    let permissions = required_permissions(object)?;

    Ok(PluginManifest {
        plugin_id: required_string(object, "plugin_id")?,
        name: required_string(object, "name")?,
        description: required_string(object, "description")?,
        author: required_string(object, "author")?,
        license: required_string(object, "license")?,
        plugin_version: required_string(object, "plugin_version")?,
        minimum_silica_version: required_string(object, "minimum_silica_version")?,
        plugin_type,
        permissions,
        entry: optional_string(object, "entry")?,
        homepage: optional_string(object, "homepage")?,
        repository: optional_string(object, "repository")?,
        enabled_by_default: false,
    })
}

fn required_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, PluginManifestError> {
    let value = object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| PluginManifestError::InvalidField(format!("{field} must be a string")))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(PluginManifestError::InvalidField(format!(
            "{field} must not be empty"
        )));
    }
    Ok(value.to_string())
}

fn optional_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<Option<String>, PluginManifestError> {
    match object.get(field) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(value)) if value.trim().is_empty() => Err(
            PluginManifestError::InvalidField(format!("{field} must not be empty when present")),
        ),
        Some(serde_json::Value::String(value)) => Ok(Some(value.trim().to_string())),
        Some(_) => Err(PluginManifestError::InvalidField(format!(
            "{field} must be a string or null"
        ))),
    }
}

fn required_permissions(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<String>, PluginManifestError> {
    let values = object
        .get("permissions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            PluginManifestError::InvalidField("permissions must be an array".to_string())
        })?;
    if values.is_empty() {
        return Err(PluginManifestError::InvalidField(
            "permissions must not be empty".to_string(),
        ));
    }

    let mut seen = BTreeSet::new();
    let mut permissions = Vec::with_capacity(values.len());
    for value in values {
        let permission = value.as_str().ok_or_else(|| {
            PluginManifestError::InvalidField("permission must be a string".to_string())
        })?;
        let permission = permission.trim();
        if permission.is_empty() {
            return Err(PluginManifestError::InvalidField(
                "permission must not be empty".to_string(),
            ));
        }
        if !PLUGIN_BOUNDARY_PERMISSION_IDS.contains(&permission) {
            return Err(PluginManifestError::ForbiddenPermission(
                permission.to_string(),
            ));
        }
        if !seen.insert(permission.to_string()) {
            return Err(PluginManifestError::InvalidField(format!(
                "duplicate permission {permission}"
            )));
        }
        permissions.push(permission.to_string());
    }

    Ok(permissions)
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-plugin");
    }

    #[test]
    fn plugin_boundary_starts_with_no_granted_permissions() {
        assert!(super::PLUGIN_DEFAULT_GRANTED_PERMISSION_IDS.is_empty());
    }

    #[test]
    fn plugin_boundary_declares_no_forbidden_permission_ids() {
        for permission_id in super::PLUGIN_BOUNDARY_PERMISSION_IDS {
            assert_ne!(*permission_id, "raw_sql");
            assert_ne!(*permission_id, "database:raw_sql");
            assert_ne!(*permission_id, "original:mutate");
        }
    }

    #[test]
    fn validates_plugin_manifest_and_keeps_plugin_disabled_by_default() {
        let manifest = super::validate_plugin_manifest_json(
            r#"{
              "schema":"silica.plugin",
              "version":1,
              "plugin_id":"silicaraw.test.preset_pack",
              "name":"Silica Test Presets",
              "description":"Data-only preset pack manifest.",
              "author":"SilicaRAW",
              "license":"MIT",
              "plugin_version":"0.1.0",
              "minimum_silica_version":"0.1.0",
              "type":"preset_pack",
              "permissions":["edit_suggestion:read","edit_suggestion:apply"],
              "entry":null,
              "homepage":null,
              "repository":null
            }"#,
        )
        .expect("valid plugin manifest");

        assert_eq!(manifest.plugin_id, "silicaraw.test.preset_pack");
        assert_eq!(manifest.plugin_type, "preset_pack");
        assert_eq!(
            manifest.permissions,
            vec!["edit_suggestion:read", "edit_suggestion:apply"]
        );
        assert!(!manifest.enabled_by_default);
    }

    #[test]
    fn rejects_plugin_manifest_missing_required_trust_fields() {
        for (label, manifest_json) in [
            (
                "license",
                r#"{
                  "schema":"silica.plugin",
                  "version":1,
                  "plugin_id":"silicaraw.test",
                  "name":"Test",
                  "description":"Missing license.",
                  "author":"SilicaRAW",
                  "plugin_version":"0.1.0",
                  "minimum_silica_version":"0.1.0",
                  "type":"preset_pack",
                  "permissions":["edit_suggestion:read"]
                }"#,
            ),
            (
                "minimum_silica_version",
                r#"{
                  "schema":"silica.plugin",
                  "version":1,
                  "plugin_id":"silicaraw.test",
                  "name":"Test",
                  "description":"Missing minimum app version.",
                  "author":"SilicaRAW",
                  "license":"MIT",
                  "plugin_version":"0.1.0",
                  "type":"preset_pack",
                  "permissions":["edit_suggestion:read"]
                }"#,
            ),
            (
                "permissions",
                r#"{
                  "schema":"silica.plugin",
                  "version":1,
                  "plugin_id":"silicaraw.test",
                  "name":"Test",
                  "description":"Missing permissions.",
                  "author":"SilicaRAW",
                  "license":"MIT",
                  "plugin_version":"0.1.0",
                  "minimum_silica_version":"0.1.0",
                  "type":"preset_pack"
                }"#,
            ),
        ] {
            let error = super::validate_plugin_manifest_json(manifest_json)
                .expect_err("missing required field must fail");
            assert!(
                error.to_string().contains(label),
                "expected {label} error, got {error}"
            );
        }
    }

    #[test]
    fn rejects_raw_sql_and_unknown_plugin_permissions() {
        for permission_id in ["raw_sql", "database:raw_sql", "filesystem:limited_write"] {
            let manifest_json = format!(
                r#"{{
                  "schema":"silica.plugin",
                  "version":1,
                  "plugin_id":"silicaraw.test",
                  "name":"Test",
                  "description":"Forbidden permission.",
                  "author":"SilicaRAW",
                  "license":"MIT",
                  "plugin_version":"0.1.0",
                  "minimum_silica_version":"0.1.0",
                  "type":"preset_pack",
                  "permissions":["{permission_id}"]
                }}"#,
            );

            let error = super::validate_plugin_manifest_json(&manifest_json)
                .expect_err("forbidden permission must fail");
            assert!(
                error.to_string().contains(permission_id),
                "expected {permission_id} error, got {error}"
            );
        }
    }
}
