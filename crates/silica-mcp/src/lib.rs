//! MCP boundary for SilicaRAW.
//!
//! Task 23.1 records permission ID boundaries only.

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-mcp";

pub const MCP_MODE_PERMISSION_IDS: &[&str] =
    &["mcp:read_only", "mcp:review", "mcp:edit", "mcp:export"];

pub const MCP_DEFAULT_GRANTED_PERMISSION_IDS: &[&str] = &[];

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

#[cfg(test)]
mod tests {
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
}
