//! Plugin boundary for SilicaRAW.
//!
//! Task 23.1 records permission ID boundaries only.

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
}
