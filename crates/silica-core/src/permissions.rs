use std::collections::BTreeSet;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-core";

/// Coarse permission category for future extension actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionPermissionCategory {
    Metadata,
    EditSuggestion,
    Export,
    Filesystem,
    AiResult,
    McpMode,
}

/// Default-deny permission vocabulary for future plugin, MCP, and AI actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionPermission {
    MetadataRead,
    MetadataWrite,
    EditSuggestionRead,
    EditSuggestionApply,
    ExportLocal,
    FilesystemLimitedRead,
    FilesystemLimitedWrite,
    AiResultRead,
    AiResultPropose,
    McpReadOnly,
    McpReview,
    McpEdit,
    McpExport,
}

impl ExtensionPermission {
    pub const ALL: [Self; 13] = [
        Self::MetadataRead,
        Self::MetadataWrite,
        Self::EditSuggestionRead,
        Self::EditSuggestionApply,
        Self::ExportLocal,
        Self::FilesystemLimitedRead,
        Self::FilesystemLimitedWrite,
        Self::AiResultRead,
        Self::AiResultPropose,
        Self::McpReadOnly,
        Self::McpReview,
        Self::McpEdit,
        Self::McpExport,
    ];

    pub fn stable_id(self) -> &'static str {
        match self {
            Self::MetadataRead => "metadata:read",
            Self::MetadataWrite => "metadata:write",
            Self::EditSuggestionRead => "edit_suggestion:read",
            Self::EditSuggestionApply => "edit_suggestion:apply",
            Self::ExportLocal => "export:local",
            Self::FilesystemLimitedRead => "filesystem:limited_read",
            Self::FilesystemLimitedWrite => "filesystem:limited_write",
            Self::AiResultRead => "ai_result:read",
            Self::AiResultPropose => "ai_result:propose",
            Self::McpReadOnly => "mcp:read_only",
            Self::McpReview => "mcp:review",
            Self::McpEdit => "mcp:edit",
            Self::McpExport => "mcp:export",
        }
    }

    pub fn category(self) -> ExtensionPermissionCategory {
        match self {
            Self::MetadataRead | Self::MetadataWrite => ExtensionPermissionCategory::Metadata,
            Self::EditSuggestionRead | Self::EditSuggestionApply => {
                ExtensionPermissionCategory::EditSuggestion
            }
            Self::ExportLocal => ExtensionPermissionCategory::Export,
            Self::FilesystemLimitedRead | Self::FilesystemLimitedWrite => {
                ExtensionPermissionCategory::Filesystem
            }
            Self::AiResultRead | Self::AiResultPropose => ExtensionPermissionCategory::AiResult,
            Self::McpReadOnly | Self::McpReview | Self::McpEdit | Self::McpExport => {
                ExtensionPermissionCategory::McpMode
            }
        }
    }

    pub fn allows_original_mutation(self) -> bool {
        match self {
            Self::MetadataRead
            | Self::MetadataWrite
            | Self::EditSuggestionRead
            | Self::EditSuggestionApply
            | Self::ExportLocal
            | Self::FilesystemLimitedRead
            | Self::FilesystemLimitedWrite
            | Self::AiResultRead
            | Self::AiResultPropose
            | Self::McpReadOnly
            | Self::McpReview
            | Self::McpEdit
            | Self::McpExport => false,
        }
    }
}

/// Future extension permission policy. Empty/default policy denies every permission.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtensionPermissionPolicy {
    grants: BTreeSet<ExtensionPermission>,
}

impl ExtensionPermissionPolicy {
    pub fn allows(&self, permission: ExtensionPermission) -> bool {
        self.grants.contains(&permission)
    }

    pub fn with_grant(mut self, permission: ExtensionPermission) -> Self {
        self.grants.insert(permission);
        self
    }
}

/// Permissioned MCP modes. `Off` has no permission because it starts no runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum McpMode {
    Off,
    ReadOnly,
    Review,
    Edit,
    Export,
}

impl McpMode {
    pub fn required_permission(self) -> Option<ExtensionPermission> {
        match self {
            Self::Off => None,
            Self::ReadOnly => Some(ExtensionPermission::McpReadOnly),
            Self::Review => Some(ExtensionPermission::McpReview),
            Self::Edit => Some(ExtensionPermission::McpEdit),
            Self::Export => Some(ExtensionPermission::McpExport),
        }
    }
}
